
# Logbot REST API

This crates provides a REST-api for controlling and monitoring logbot.

## Endpoints

Available endpoints are:

- `/v1/demo`: Demo
- `/v1/calibrate`: Calibrate
- `/v1/edge`: Find the edge of the line
- `/v1/follow`: Follow the line
- `/v1/stop`: Stop

There's a also a health check endpoint, but responses have no body and it only
replied with HTTP Status Code 200 when the service is healthy.

- `/v1/health`: Health Check

The REST endpoints should be called with a HTTP POST request.
The JSON structure for responses are:

```json
{
  "status": int,
  "reason": string/null,
}
```

The status field holds an integer with has the HTTP Status Code naming convention.
Here are a few examples of responses:

```json
{
  "status": 200,
  "reason": null,
}
```

```json
{
  "status": 405, // Method not allowed
  "reason": "NOCALIBRATION",
}
```
