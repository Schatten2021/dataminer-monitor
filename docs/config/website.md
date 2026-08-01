Website configuration
---------------------
| field    | type                                                    | description                                                                                    |
|----------|---------------------------------------------------------|------------------------------------------------------------------------------------------------|
| url      | Url                                                     | The url to request (with method)                                                               |
| interval | Duration                                                | The interval in which to request the url.                                                      |
| status   | [SingleFilter](filter.md#single-filter) of status-codes | A Filter to apply to the returned status codes. 200-299 codes are accepted unless blacklisted. |


# Example
```toml
[website.foo]
url = "https://example.com/"
# Note: The specification is this way due to `chrono`s Duration serialization.
# Note: The first value is the seconds the second on ehte nanoseconds.
interval = [60, 0]
status.accept = [200]
```