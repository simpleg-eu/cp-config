# Endpoints

## config

### `GET`

Query parameters:

* `stage`: for now, git branch from which the configuration will be retrieved.
* `environment`: environment from which the configuration will be retrieved.
* `component`: component from which the configuration will be retrieved.

Responses:

* `200`: Package file containing the expected configuration files.
* `400`: Environment or component could not be found.
* `500`: Server failure.

## healthz/ready

### `GET`

Responses:

* `200`: Configuration provider is ready.
* `500`: Not ready.

## healthz/live

### `GET`

Responses:

* `200`: Configuration provider is alive.