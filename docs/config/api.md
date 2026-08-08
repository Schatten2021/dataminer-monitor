API configuration
-----------------

| field             | type                                                                                     | description                                           |
|-------------------|------------------------------------------------------------------------------------------|-------------------------------------------------------|
| path              | String                                                                                   | The prefix of the API paths. **MIND THE LEADING '/'** |
| filter-attributes | [SingleFilter](filter.md#single-filter) of [AttributeChange](filter.md#attributematcher) | Filter the attributes to be displayed on the website  |
| filter-elements   | [SingleFilter](filter.md#single-filter) of Strings                                       | Filters the Elements based on their IDs               |

## Example
```toml
[api]
path = "/api" # the default path
filter-attributes = ["minecraft.players"] # do not display online player-count/players
```