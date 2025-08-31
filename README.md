# Dataminer Monitor
This is my own rocket-based monitor server for my data-miners.

It is built to be easily integratable into any dataminer. 
Because of this the only way the monitor interacts with the dataminer is through pings from the dataminer.

# Configuration
Note: The entire `[email]` config section is required right now. 
In order to run the server, the `config.toml` file **must** contain a correctly formatted `[email]` section.

The configuration file (`config.toml` by default) has the following structure:

## `notify`
- This sets the E-Mails that are to be notified when a dataminer goes on- or offline.
- It is formatted as a list of strings (e.g. `notify = ["john.doe@example.com"]`)

## `[email]`
This subsection dictates the parameters for the E-Mail-Account which will be used to send the notifications.
It has the following (hopefully self-explanatory) fields:
- `address`
- `password`
- `server`
## `[miners]`
This section dictates the configuration for each dataminer.

Note: The only value currently defined is `timeout`, which is in the format of `[seconds, nanoseconds]` (due to chrono serialization).

A dataminer might be configured like this: 
```toml
[miner.example]
timeout = [60, 0] # Timeout after 60 seconds
```

## rocket
This server uses [rockets](https://rocket.rs/) default [Figment](https://crates.io/crates/figment) configuration (see [the rocket documentation](https://rocket.rs/guide/master/configuration/)).

The ideal way to configure it is probably via `Rocket.toml`
