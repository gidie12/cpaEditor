# ProcessSpecification

## Attributes
- `name`: The name of the process specification.
- `version`: The version of the process specification.
- `uuid`: A unique identifier (optional).

## Sub-elements
- `ds:Reference` (optional, unbounded)

## Example
```xml
<ProcessSpecification name="OrderProcess" version="1.0" uuid="uuid-1234">
    <ds:Reference URI="#ref1"/>
</ProcessSpecification>
```