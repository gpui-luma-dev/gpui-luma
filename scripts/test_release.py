"""Offline failure-path tests; these never invoke Cargo publishing or crates.io."""

import copy
import json
import os
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch
import urllib.error

import release


class ReleaseTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.report_path = Path(self.temporary.name) / 'status.json'
        patcher = patch.object(release, 'REPORT', self.report_path)
        patcher.start()
        self.addCleanup(patcher.stop)
        patcher = patch.dict(os.environ, {'CARGO_REGISTRY_TOKEN': 'secret-token'}, clear=True)
        patcher.start()
        self.addCleanup(patcher.stop)
        self.report = {'version': '0.1.9', 'commit': 'abc', 'phase': 'preflight', 'error': None,
                       'packages': {'sdk': {'status': 'pending'}, 'plot': {'status': 'pending'}}}

    @patch.object(release, 'download')
    def test_new_crate_blocks_before_upload(self, download):
        download.side_effect = [b'{}', None, None, None]
        with patch.object(release.subprocess, 'run') as run:
            with self.assertRaisesRegex(release.ReleaseError, 'No crates uploaded'):
                release.preflight(self.report, publish=True, allow_new=False)
            run.assert_not_called()
        self.assertTrue(json.loads(self.report_path.read_text())['packages']['plot']['new_crate'])

    @patch.object(release, 'download', return_value=None)
    def test_verify_only_needs_no_token(self, download):
        del os.environ['CARGO_REGISTRY_TOKEN']
        release.preflight(self.report, publish=False, allow_new=False)

    @patch.object(release, 'download', return_value=None)
    def test_missing_token_blocks_before_upload(self, download):
        del os.environ['CARGO_REGISTRY_TOKEN']
        with self.assertRaisesRegex(release.ReleaseError, 'Missing CARGO_REGISTRY_TOKEN'):
            release.preflight(self.report, publish=True, allow_new=True)

    def test_permission_failure_preserves_partial_status_and_redacts_log(self):
        with patch.object(release, 'preflight'), patch.object(release, 'local_contents', return_value={}), \
                patch.object(release, 'verify', side_effect=[True, False]), \
                patch.object(release.subprocess, 'run', return_value=subprocess.CompletedProcess(
                    [], 101, '403 Forbidden secret-token')) as run:
            with self.assertRaisesRegex(release.ReleaseError, 'token rejected'):
                release.publish_packages(self.report, True, False)
            self.assertEqual(run.call_count, 1)
        state = json.loads(self.report_path.read_text())
        self.assertEqual(state['packages']['sdk']['status'], 'verified')
        self.assertEqual(state['packages']['plot']['status'], 'permission denied')
        self.assertNotIn('secret-token', (self.report_path.parent / 'plot.log').read_text())

    def test_timeout_after_successful_upload_is_verified_without_reupload(self):
        with patch.object(release, 'preflight'), patch.object(release, 'local_contents', return_value={}), \
                patch.object(release, 'verify', side_effect=[True, False, True]), \
                patch.object(release.subprocess, 'run', return_value=subprocess.CompletedProcess(
                    [], 101, 'index timeout')) as run:
            release.publish_packages(self.report, True, False)
            self.assertEqual(run.call_count, 1)
        self.assertTrue(all(state['status'] == 'verified' for state in self.report['packages'].values()))

    def test_cargo_process_timeout_checks_registry_before_stopping(self):
        with patch.object(release, 'preflight'), patch.object(release, 'local_contents', return_value={}), \
                patch.object(release, 'verify', side_effect=[True, False, True]), \
                patch.object(release.subprocess, 'run', side_effect=subprocess.TimeoutExpired(
                    'cargo', 600, output=b'waiting for index')) as run:
            release.publish_packages(self.report, True, False)
            self.assertEqual(run.call_count, 1)
        self.assertEqual(self.report['packages']['plot']['status'], 'verified')

    def test_dirty_or_wrong_commit_archive_is_rejected(self):
        archive_path = Path(self.temporary.name) / 'sdk-0.1.9.crate'
        archive_path.write_bytes(b'archive')
        for vcs in ({'sha1': 'different'}, {'sha1': 'abc', 'dirty': True}):
            with self.subTest(vcs=vcs), patch.object(release, 'Path', return_value=archive_path.parent), \
                    patch.object(release, 'contents', return_value={
                        'sdk-0.1.9/.cargo_vcs_info.json': json.dumps({'git': vcs}).encode()}):
                with self.assertRaisesRegex(release.ReleaseError, 'clean release commit'):
                    release.local_contents('sdk', self.report)

    def test_unverified_upload_stops_without_blind_retry(self):
        with patch.object(release, 'preflight'), patch.object(release, 'local_contents', return_value={}), \
                patch.object(release, 'verify', return_value=False), \
                patch.object(release.subprocess, 'run', return_value=subprocess.CompletedProcess(
                    [], 101, 'network failure')) as run:
            with self.assertRaisesRegex(release.ReleaseError, 'No automatic upload retry'):
                release.publish_packages(self.report, True, False)
            self.assertEqual(run.call_count, 1)
        self.assertEqual(self.report['packages']['plot']['status'], 'pending')

    def test_matching_existing_crates_are_skipped_on_resume(self):
        with patch.object(release, 'preflight'), patch.object(release, 'local_contents', return_value={}), \
                patch.object(release, 'verify', return_value=True), patch.object(release.subprocess, 'run') as run:
            release.publish_packages(self.report, True, False)
            run.assert_not_called()

    def test_mismatch_blocks_all_uploads(self):
        with patch.object(release, 'preflight'), patch.object(release, 'local_contents', return_value={}), \
                patch.object(release, 'verify', side_effect=release.ReleaseError('published contents differ')), \
                patch.object(release.subprocess, 'run') as run:
            with self.assertRaisesRegex(release.ReleaseError, 'contents differ'):
                release.publish_packages(self.report, True, False)
            run.assert_not_called()

    def test_missing_archive_blocks_all_uploads(self):
        with patch.object(release, 'preflight'), \
                patch.object(release, 'local_contents', side_effect=[{}, release.ReleaseError('missing archive')]), \
                patch.object(release.subprocess, 'run') as run:
            with self.assertRaisesRegex(release.ReleaseError, 'missing archive'):
                release.publish_packages(self.report, True, False)
            run.assert_not_called()

    def test_verify_only_does_not_upload(self):
        with patch.object(release, 'preflight'), patch.object(release, 'local_contents', return_value={}), \
                patch.object(release, 'verify', return_value=False), patch.object(release.subprocess, 'run') as run:
            release.publish_packages(self.report, False, False)
            run.assert_not_called()
        self.assertIn('verified only', self.report['phase'])

    @patch.object(release.time, 'sleep')
    @patch.object(release.urllib.request, 'urlopen')
    def test_transient_registry_failure_retries_and_is_not_missing(self, urlopen, sleep):
        urlopen.side_effect = urllib.error.HTTPError('url', 503, 'unavailable', {}, None)
        with self.assertRaisesRegex(release.ReleaseError, 'unavailable after'):
            release.download('https://example.test', missing_ok=True)
        self.assertEqual(urlopen.call_count, 4)

    @patch.object(release.urllib.request, 'urlopen')
    def test_registry_403_is_not_missing_or_retried(self, urlopen):
        urlopen.side_effect = urllib.error.HTTPError('url', 403, 'forbidden', {}, None)
        with self.assertRaisesRegex(release.ReleaseError, 'HTTP 403'):
            release.download('https://example.test', missing_ok=True)
        self.assertEqual(urlopen.call_count, 1)

    def test_mismatched_archive_is_rejected(self):
        with patch.object(release, 'download', return_value=b'archive'), \
                patch.object(release, 'contents', return_value={'source': b'changed'}):
            with self.assertRaisesRegex(release.ReleaseError, 'published contents differ'):
                release.verify('sdk', self.report, {'source': b'original'})

    def test_summary_retains_recovery_information(self):
        report = copy.deepcopy(self.report)
        report.update(phase='failed', error='token rejected')
        release.save(report)
        summary_path = Path(self.temporary.name) / 'summary.md'
        with patch.dict(os.environ, {'GITHUB_STEP_SUMMARY': str(summary_path)}):
            release.summary()
        self.assertIn('token rejected', summary_path.read_text())
        self.assertIn('re-run the original commit', summary_path.read_text())


if __name__ == '__main__':
    unittest.main()
