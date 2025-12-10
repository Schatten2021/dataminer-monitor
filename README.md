# Dataminer Monitor
This is my own rocket-based monitor server for my data-miners.

It is built to be easily integratable into any dataminer. 
Because of this the only way the monitor interacts with the dataminer is through pings from the dataminer.

The entire system works around the concept of a "provider".

There are two basic "providers": `StatusProvider` & `NotificationProvider`.

These share the task of detecting when a server/service is down (`StatusProvider`) & sending out notifications about the outage (`NotificationProvider`).

There are currently 2 `StatusProvider` and 2 `NotificationProvider`:
- `miner` (`StatusProvider`, receives pings, made for dataminer)
- `webserver` (`StatusProvider`, sends pings, **only compatible with HTTP servers that support GET**)
- `email` (`NotificationProvider`, sends out E-Mails when a service goes on-/offline)
- `webserver` (`NotificationProvider`, provides the front-end for the server)

Note: `miners` are only able to expose routes that are part of their specific subroute (e.g. `/miner` for the Dataminer builtin).
**This means that to ping the dataminer one must request `/miner/ping?id={id}`**

# Configuration
There are two basic configuration keys in the configuration file (`config.toml`): `status` & `notifications`.

Each key controls the configuration for its providers (`status` for all `StatusProvider` & `notifications` for all `NotificationProvider`).

The different fields for each provider are accessible via `status.provider-id.*` (Note: `provider-id` is the `NotificationProvider::ID`) or `notifications.provider-id.*` (Note: similar)

## Feature flags
Some configuration (especially selection of default providers) are done via Rust's feature flags.
These can be activated/deactivated via `--no-default-features --features <foo,bar>` when building/running via cargo (e.g. `cargo build --no-default-features --features frontend-website frontend-websocket`)

The feature flags are the following:

| flag                       | what does it toggle?                                                  |
|----------------------------|-----------------------------------------------------------------------|
| `e-mail-notifications`     | the support sending E-Mails                                           |
| `frontend-website`         | The web frontend                                                      |
| `frontend-websocket`       | The websocket (for live updates; no this isn't in `frontent-website`) |
| `data-miner-status-source` | Whether dataminers as a status source are supported                   |
| `server-status-source`     | The website monitoring                                                |

Then there are the `all-notifications` & `all-data-sources` flags which I think are pretty self-explanatory.

## Builtins configuration
### `StatusProvider`
#### Dataminer
Dataminers are configurable under the `status.miner` key.

They have the following fields:

| field           | type             | description                                                                                                                            |
|-----------------|------------------|----------------------------------------------------------------------------------------------------------------------------------------|
| timeout         | [number, number] | This is the duration after which the miner will timout (format [seconds, nanoseconds] due to [chrono](https://crates.io/crates/chrono) |
| name (optional) | string           | The name under which the miner will be referenced in notifications                                                                     |

An example miner:
```toml
[status.miner.foo]
timeout = [10, 0] # Timeout after 10s
name = "Foo"
```

### Websites
Websites are configurable via the `status.webserver` key.

They have the following fields:

| field                     | type             | description                                                                                                                            | 
|---------------------------|------------------|----------------------------------------------------------------------------------------------------------------------------------------|
| url                       | string           | The url which will get pinged (Note: **This url must support GET**)                                                                    |
| interval                  | [number, number] | The interval in which to ping the server (format see Dataminer::timeout)                                                               |
| name (optional)           | string           | see Dataminer::name                                                                                                                    |
| accepted_stati (optional) | [number,...]     | A list of expected status codes. This can be used to overwrite the accepted status codes (i.e. for sites requiring login adding a 401) |
| rejected_stati (optional) | [number,...]     | Similar to expected_status. Defaults to the range of 400 - 599 (client error + Server error). Note that expected_status has priority.  | 

An example website:
```toml
[status.webserver.foo]
url = "https://login.example.com/"
interval = [3600, 0]
name = "Example website"
accepted_stati = [401]
rejected_stati = [200]
```

### Minecraft
Minecraft java servers are configurable via the `status.minecraft.java` key.

They have the following fields:

| field               | type             | description                                                               |
|---------------------|------------------|---------------------------------------------------------------------------|
| url                 | string           | The url which will get pinged                                             |
| port (optional)     | number           | The port the server is running on. (defaults to 25565)                    |
| interval (optional) | [number, number] | The interval in which to ping the server (format see Dataminer::timeout,) |
| name (optional)     | string           | see Dataminer::name                                                       |

An example Minecraft java server:
```toml
[status.minecraft.java.example]
url = "example.com"
port = 25565
interval = [3600, 0]
name = "Example minecraft server"
```
usually:
```toml
[status.minecraft.java.example]
url = "example.com"
name = "Example minecraft server"
```

## `NotificationProvider`
### E-Mail
E-Mail notifications are configurable under `notifications.email`.

They have the following fields:

| field       | type                    | description                                                                                                                                 | 
|-------------|-------------------------|---------------------------------------------------------------------------------------------------------------------------------------------|
| address     | string                  | The E-Mail address to be used when sending E-Mails (Note: Must be the same as when logging in into the E-Mail-Server                        |
| password    | string                  | The E-Mail password for the account (yes this is currently plaintext)                                                                       | 
| server      | string                  | The Address of the E-Mail-Server that is connected to (this is currently not read from the E-Mail)                                          |
| whitelist   | [string, ...] optional  | Notification types to send (Custom, Seen, WentOnline or WentOffline). Any other types will not be sent. Mutually exclusive with `blacklist` |
| blacklist   | [string, ...] optional  | Notification types _not_ to send. Opposite of and mutually exclusive with `whitelist`. Any other types _will_ be sent.                      |
| subscribers | [string or custom, ...] | A list of the E-Mail-Addresses that are to be notified of changes                                                                           |

A "custom" subscriber has the following fields:

| field       | type     | description                                                               |
|-------------|----------|---------------------------------------------------------------------------|
| email       | string   | The E-Mail-Address that a notification will be sent to                    |
| whitelist   | [string] | Which notifications to send. See `[notifications.email.whitelist]`        |
| blacklist   | [string] | Which notification types _not_ to send. `[notifications.email.blacklist]` |

### Website
The Website currently only has one configuration: `notifications.website.static_dir`.
This controls the location for the hot-reload attempts (default is "static/" which works with this repo).

### ntfy
The ntfy configuration accepts a list of different targets. 
Each target has the same structure as the JSON request (see [ntfy docs](https://docs.ntfy.sh/publish/#publish-as-json); NOTE: actions aren't supported).

In addition to the ntfy json fields are the following fields:

| field      | type     | description                                                                        |
|------------|----------|------------------------------------------------------------------------------------|
| base       | url      | The base url to which to send the notifications to (baseurl of the ntfy server)    |
| whitelist  | [string] | see `[email.whitelist]`                                                            |
| blacklist  | [string] | see `[email.blacklist]`                                                            |
| auth_token | string   | the authentication  (see [ntfy docs](https://docs.ntfy.sh/publish/#access-tokens)) |

## Rocket
Rocket is configured the same way that any rocket server would be, via a `Rocket.toml` file.
For more details please consult the [rocket documentation](https://rocket.rs/guide/master/configuration/).

## Disabling providers
You can manually disable StatusProvider & NotificationProvider via the config.
This makes it so that they are also not present in the api or the website without requiring recompilation.

You can disable certain providers by adding them in the `[disabled]` section of the config.
(Note: add them to `disabled.status` or `disabled.notifications`)

Example:
```toml
[disabled]
status = ["dataminer"]
notifications = ["email"]
```