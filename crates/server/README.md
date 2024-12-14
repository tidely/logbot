
# Logbot REST API

This crates provides a REST-api for controlling and monitoring logbot.

## Endpoints

Endpoints with hardware controlling abilities should be called using HTTP POST requests.

Available endpoints are:

- `/v1/demo`: Demo
- `/v1/calibrate`: Calibrate
- `/v1/edge`: Find the edge of the line
- `/v1/follow`: Follow the line
- `/v1/stop`: Stop

There is a Health Check endpoint, which should be called using a HTTP GET request.

- `/v1/health`: Health Check

All endpoints return JSON in the response body. The structure of responses are as follows:

```json
{
  "status": int,
  "reason": string,
}
```

The status field holds an integer with has the HTTP Status Code naming convention. When a request is successful, the `reason` field depicts the action that was cancelled by this request. On a failure, the field describes the reason for failure. The `Health` endpoint is an exception. This always returns `Health`.


### Examples

Here is an example of requests send to the API with the given responses:

```sh
curl -X POST http://127.0.0.1:9999/v1/calibrate
```

```json
{
  "status": 200,
  "reason": "Stop", // When starting to move, we consider it cancelling a Stop call
}
```

```sh
curl -X POST http://127.0.0.1:9999/v1/demo
```

```json
{
  "status": 409,
  "reason": "Calibrate", // We are busy with calibrating
}
```

```sh
curl -X GET http://127.0.0.1:9999/v1/health
```

```json
{
  "status": 200,
  "reason": "Health", // Health endpoint always returns Health
}
```

```sh
curl -X POST http://127.0.0.1:9999/v1/follow
```

```json
{
  "status": 403, // FORBIDDEN
  "reason": "FindEdge", // We need to call FindEdge before we can follow the line
}
```

```sh
curl -X POST http://127.0.0.1:9999/v1/stop
```

```json
{
  "status": 200,
  "reason": "Calibrate", // We cancelled the calibration before it finished
}
```
