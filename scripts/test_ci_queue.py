"""Exercise the workflow's inline queue guard without network or sleeping."""

import io
import json
import os
from pathlib import Path
import textwrap
import unittest
from unittest.mock import patch
import urllib.error


class QueueGuardTests(unittest.TestCase):
    def run_guard(self, states, clock=(0, 1)):
        workflow = (Path(__file__).resolve().parents[1] / '.github/workflows/ci.yml').read_text()
        script = textwrap.dedent(workflow.split('        shell: python\n        run: |\n', 1)[1])
        calls = []
        states = iter(states)

        def urlopen(request, timeout):
            calls.append((request.full_url, request.method))
            self.assertEqual(timeout, 30)
            if request.method == 'POST':
                return io.BytesIO(b'')
            state = next(states)
            if isinstance(state, Exception):
                raise state
            jobs = [] if state is None else [{'name': 'Test & Doc-tests', 'status': state}]
            return io.BytesIO(json.dumps({'jobs': jobs}).encode())

        environment = {
            'GITHUB_API_URL': 'https://api.github.com',
            'GITHUB_REPOSITORY': 'owner/repo', 'GITHUB_RUN_ID': '123',
            'GITHUB_RUN_ATTEMPT': '2', 'GH_TOKEN': 'test-token',
        }
        with patch.dict(os.environ, environment), patch('urllib.request.urlopen', side_effect=urlopen), \
             patch('time.monotonic', side_effect=clock), patch('time.sleep') as sleep, \
             patch('sys.stdout', new_callable=io.StringIO):
            try:
                exec(compile(script, 'ci-queue-guard', 'exec'), {})
            except SystemExit as error:
                return calls, sleep.call_count, error.code
        return calls, sleep.call_count, 0

    def test_running_or_completed_job_passes_without_cancelling(self):
        for state in ('in_progress', 'completed'):
            calls, sleeps, code = self.run_guard([state])
            self.assertEqual((len(calls), sleeps, code), (1, 0, 0))
            self.assertIn('/attempts/2/jobs?', calls[0][0])

    def test_waits_for_missing_and_queued_job(self):
        calls, sleeps, code = self.run_guard([None, 'queued', 'in_progress'], clock=(0, 1, 31))
        self.assertEqual((len(calls), sleeps, code), (3, 2, 0))

    def test_queue_deadline_cancels_run_and_fails(self):
        calls, sleeps, code = self.run_guard(['queued'], clock=(0, 900))
        self.assertEqual((sleeps, code), (0, 1))
        self.assertEqual(calls[-1], ('https://api.github.com/repos/owner/repo/actions/runs/123/cancel', 'POST'))

    def test_api_failure_does_not_report_success(self):
        with self.assertRaises(urllib.error.URLError):
            self.run_guard([urllib.error.URLError('unavailable')])


if __name__ == '__main__':
    unittest.main()
