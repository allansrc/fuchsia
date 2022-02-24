#!/usr/bin/env python3.8
# Copyright 2019 The Fuchsia Authors. All rights reserved.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

import argparse
import os
import stat
import string
import sys


def main():
    parser = argparse.ArgumentParser(
        description='Generates a grouped dart test file from individual tests')
    parser.add_argument(
        '--out', help='Path to the invocation file to generate', required=True)
    parser.add_argument(
        '--source',
        help=
        'Path to a dart source file. Will be ignored if the file does not end in _test.dart',
        action='append',
        required=True)
    args = parser.parse_args()

    grouped_test = args.out
    grouped_test_dir = os.path.dirname(grouped_test)
    if not os.path.exists(grouped_test_dir):
        os.makedirs(grouped_test_dir)

    tests = [src for src in args.source if src.endswith('_test.dart')]
    assert len(
        tests
    ) > 0, 'a least one *_test.dart file must be passed in under |sources|'
    imports = ["import 'package:test/test.dart';"]
    invocations = []
    for test in tests:
        filename = os.path.splitext(os.path.basename(test))[0]
        imports.append("import '%s' as %s;" % (test, filename))
        invocations.append(
            "group('%s', () => _delegate(%s.main, args));" %
            (filename, filename))

    contents = '''// DO NOT EDIT
// This script is generated by:
//   //build/dart/group_tests.py

%s

typedef ZeroArgFunction = void Function();

/// Wraps main invocations to support both main(List<String> args) and main().
void _delegate(Function f, List<String> args) {
  if (f is ZeroArgFunction) {
    f();
  } else {
    f(args);
  }
}

void main(List<String> args) {
  %s
}''' % ('\n'.join(imports), '\n  '.join(invocations))

    with open(grouped_test, 'w') as file:
        file.write(contents)
    permissions = (
        stat.S_IRUSR | stat.S_IWUSR | stat.S_IXUSR | stat.S_IRGRP |
        stat.S_IWGRP | stat.S_IXGRP | stat.S_IROTH)
    os.chmod(grouped_test, permissions)


if __name__ == '__main__':
    sys.exit(main())