Dataminer configuration
-----------------------
- A Map of miner ids → miner configuration

| field   | type     | description                                |
|---------|----------|--------------------------------------------|
| timeout | Duration | How long to wait until the miner times out |


# Example
```toml
[miner.foo]
# Note: The specification is this way due to `chrono`s Duration serialization.
# Note: The first value is the seconds the second on ehte nanoseconds.
timeout = [5, 0]
```