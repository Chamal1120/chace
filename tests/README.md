## Running the unit tests

```bash
cargo test --bins
```

## Running the integration Tests

00. Set up API keys:
```bash
export GEMINI_API_KEY="your-key-here"
export GROQ_API_KEY="your-key-here"
```
01. Export the test socket path:
```bash
export SOCKET_PATH="/tmp/chace_test.sock"
```

02. In one terminal, start the server:
```bash
cargo run --release
```

03. In another terminal, run the tests:
```bash
cargo test --test socket_integration_tests -- --test-threads=1
```

> [!NOTE]
> Tests are run with `--test-threads=1` to avoid socket conflicts.

## Test Structure

Each test:
1. Waits for the socket to be available (with timeout)
2. Connects to the Unix socket
3. Sends a JSON request
4. Validates the response structure
5. Checks for expected success or error conditions

## Additional Notes

- Some tests (like `test_rust_empty_function_success`) require valid API keys
- The socket path is `/tmp/chace_test.sock` for tests (vs `/tmp/chace.sock` for production)
