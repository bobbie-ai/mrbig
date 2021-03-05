# gRPC tests

The tests in this folder must depend only on `mrbig*` crates and eventually other `tonic`'s dependencies, such as `tokio`, `futures` etc.

*(must not depend directly on `grpc_reflection*` or others)*

## Requirements

Install [`grpc_cli`](https://github.com/grpc/grpc/blob/master/doc/command_line_tool.md#code-location) and make sure it is in your `$PATH` before exectuting the tests.

## Testing

To run the tests:
```sh
bash test.sh
```
from within this folder.
