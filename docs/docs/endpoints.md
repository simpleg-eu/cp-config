# Endpoints

## config

### `GET`

Query parameters:

* `environment`: environment from which the configuration will be retrieved.
* `component`: component from which the configuration will be retrieved.

Responses:

* `200`: Package file containing the expected configuration files.
* `400`: Environment or component could not be found.
* `500`: Server failure.