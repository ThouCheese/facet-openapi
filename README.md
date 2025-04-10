Run the bastard like this:

```rust
#[derive(facet_derive::Facet)]
struct ApiResponse {
    status_message: String,
    payload: ApiResponsePayload,
}

#[derive(facet_derive::Facet)]
struct ApiResponsePayload {
    payload_was_useful: bool,
}

println!(
    "{}",
    serde_json::to_string_pretty(&schema::<ApiResponse>()).unwrap()
);
```

And you will see:

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "ApiResponse",
  "type": "object",
  "required": [
    "status_message",
    "payload"
  ],
  "properties": {
    "payload": {
      "$ref": "#/definitions/ApiResponsePayload"
    },
    "status_message": {
      "type": "string"
    }
  },
  "definitions": {
    "ApiResponsePayload": {
      "type": "object",
      "required": [
        "payload_was_useful"
      ],
      "properties": {
        "payload_was_useful": {
          "type": "boolean"
        }
      }
    }
  }
}
```
