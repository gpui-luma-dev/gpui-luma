"""Run a check silently on success; print its complete output on failure."""

import subprocess
import sys
import tempfile


def main():
    command = sys.argv[1:]
    if not command:
        sys.exit('Usage: quiet_check.py COMMAND [ARGS...]')
    with tempfile.TemporaryFile(mode='w+b') as output:
        try:
            result = subprocess.run(command, stdout=output, stderr=subprocess.STDOUT)
        except OSError as error:
            sys.exit(str(error))
        if result.returncode:
            print('Failed: ' + ' '.join(command), file=sys.stderr, flush=True)
            output.seek(0)
            while chunk := output.read(65536):
                sys.stderr.buffer.write(chunk)
            sys.stderr.buffer.flush()
        return result.returncode if result.returncode >= 0 else 128 - result.returncode


if __name__ == '__main__':
    sys.exit(main())
