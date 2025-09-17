# CS-CLI Regression Test Suite

Comprehensive regression test suite for the CS-CLI tool, ensuring continued functionality against the real Gong API.

## Test Categories

### 1. Unit Tests
- **HTML Processing** (`html_test.rs`) - Tests HTML to Markdown conversion
- **Performance** (`performance_test.rs`) - Tests concurrency, rate limiting, memory usage

### 2. Integration Tests (Real API)
- **Authentication** (`auth_integration.rs`) - Tests browser cookie extraction and Gong auth flow
- **API Integration** (`api_integration.rs`) - Tests real Gong API endpoints
- **E2E Regression** (`e2e_regression.rs`) - Tests complete workflows

### 3. Test Utilities
- **Common Helpers** (`common/mod.rs`) - Shared fixtures, mocks, and test utilities

## Running Tests

### Quick Test Run (Mocked)
```bash
cargo test
```

### Full Regression Suite
```bash
./run_tests.sh
```

### Real API Integration Tests
```bash
# Requires active Gong session in browser
export USE_REAL_API=true
cargo test -- --ignored --nocapture
```

### Specific Test Categories
```bash
# Unit tests only
cargo test --lib

# Integration tests only
cargo test --test '*integration*' -- --ignored

# Performance tests
cargo test performance_test

# With debug output
RUST_LOG=cs_cli=debug cargo test -- --nocapture
```

## Test Configuration

### Environment Setup
Copy the example environment file and configure for your testing needs:
```bash
cp .env.example .env
# Edit .env with your test configuration
```

### Environment Variables
- `USE_REAL_API` - Set to `true` to run against real Gong API
- `TEST_CUSTOMER_NAME` - Customer name for testing (default: "Fiserv")
- `TEST_DAYS_BACK` - Number of days to look back (default: 30)
- `RUST_LOG` - Logging level (e.g., `cs_cli=debug`)

These variables can be set in your `.env` file or overridden on the command line.

### Prerequisites for Real API Tests
1. **Browser Session**: Must be logged into Gong in Safari, Chrome, or Firefox
2. **Valid Account**: Need access to a Gong workspace with data
3. **Test Data**: Customer specified in `TEST_CUSTOMER_NAME` should exist

## Test Coverage Areas

### Authentication Flow
✅ Multi-browser cookie extraction (Safari, Chrome, Firefox, Edge, Brave)
✅ CSRF token management
✅ Workspace ID extraction
✅ Session management
✅ Error handling (expired cookies, no cookies)

### API Operations
✅ Customer search
✅ Timeline extraction (calls & emails)
✅ Email enhancement
✅ Library search
✅ Concurrent requests with rate limiting
✅ Retry logic with exponential backoff
✅ Empty results handling
✅ Large dataset pagination

### Data Processing
✅ HTML to Markdown conversion
✅ Email filtering (BDR/template detection)
✅ Call filtering by duration
✅ Similarity detection
✅ Output formatting

### End-to-End Workflows
✅ Complete extraction workflow
✅ CLI argument parsing
✅ Interactive mode structure
✅ Output directory creation
✅ Markdown file generation
✅ Error recovery

### Performance & Concurrency
✅ Semaphore-based rate limiting
✅ Connection pool efficiency
✅ Memory usage with large datasets
✅ Exponential backoff timing
✅ Task cancellation
✅ Thread safety

## Known Working Behaviors (Regression Points)

These are the critical behaviors that must continue working:

1. **Cookie Authentication**: Extract cookies from user's browser session
2. **Cell Detection**: Determine Gong cell from cookie domain
3. **CSRF Token**: Fetch and refresh CSRF tokens for API calls
4. **Rate Limiting**: Handle 429 responses gracefully with retry
5. **Concurrent Calls**: Use semaphore to limit concurrent API requests
6. **HTML Conversion**: Preserve formatting when converting emails
7. **Output Structure**: Create `ct_<customer>/` folders with markdown files

## Adding New Tests

### Unit Test Template
```rust
#[test]
fn test_new_functionality() {
    // Arrange
    let input = TestFixtures::sample_data();

    // Act
    let result = function_under_test(input);

    // Assert
    assertions::assert_valid_result(&result);
}
```

### Integration Test Template
```rust
#[tokio::test]
#[ignore = "Requires real Gong API access"]
async fn test_api_functionality() {
    let config = TestConfig::from_env();

    if !config.use_real_api {
        println!("Skipping - USE_REAL_API not set");
        return;
    }

    // Test implementation
}
```

## Continuous Integration

The test suite is designed to run in CI/CD pipelines:

```yaml
# Example GitHub Actions workflow
test:
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v2
    - run: cargo test
    - run: cargo clippy -- -D warnings
    - run: cargo fmt -- --check
```

## Troubleshooting

### Common Issues

1. **"No cookies found"** - Ensure you're logged into Gong in your browser
2. **"Rate limited"** - Tests hitting API limits, reduce concurrency
3. **"Customer not found"** - Update `TEST_CUSTOMER_NAME` to valid customer
4. **"Timeout"** - Network issues or API slowness, increase timeout

### Debug Mode
```bash
# Maximum debug output
RUST_LOG=cs_cli=trace RUST_BACKTRACE=full cargo test -- --nocapture
```

## Maintenance

### Regular Tasks
- ✅ Run full test suite before releases
- ✅ Update test data when API changes
- ✅ Add tests for new features
- ✅ Monitor test performance
- ✅ Keep mock data synchronized with real API

### Test Data Management
- Fixtures in `tests/fixtures/` for static test data
- Use `TestFixtures` struct for consistent test data
- Mock servers for API testing without real credentials

## Performance Benchmarks

Run performance benchmarks:
```bash
cargo bench
```

Expected performance targets:
- Authentication: < 2 seconds
- Customer search: < 1 second
- Timeline extraction (30 days): < 10 seconds
- HTML conversion: < 100ms per email
- Concurrent requests: 10+ req/sec with rate limiting