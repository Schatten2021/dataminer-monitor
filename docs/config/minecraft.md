Minecraft configuration
-----------------------
Minecraft currently only supports java via the `minecraft-java` config value.

`minecraft.java` is a map of server-id -> server-config

| field    | type     | description                                                    |
|----------|----------|----------------------------------------------------------------|
| url      | string   | The url to which to connect to.                                |
| port     | u16      | The port to which to connect to.                               |
| interval | Duration | The interval in which to ping the server to update the status. |


# Example
```toml
[minecraft.java.foo]
url = "exaple.com"
#port = 25565 # usually not necessary; 25565 is the default java port.
# Note: The specification is this way due to `chrono`s Duration serialization.
# Note: The first value is the seconds the second on ehte nanoseconds.
interval = [5, 0]
```