# Rusty Scheduler

Job scheduler made with [Rust](https://www.rust-lang.org).

This is a basic scheduler with job dependency support using a pipeline flow.

## Usage

### Compile

This project uses [Rust](https://www.rust-lang.org) and Cargo as the build tool.

To create a release binary:

```sh
cargo build --release
```

### Run

You can run (for development purposes) using Cargo:

```sh
cargo run -- --log trace --pipelines "./examples" --refresh 5
```

To run the binary version:

```sh
./rusty-scheduler --log error --pipelines "./pipelines" --refresh 60
```

### Settings

- `--log <trace|info|warn|error>`: Log level to use. Trace generates a lot of useful messages for development and debugging.
- `--pipelines <dir>`: Directory for all pipelines. Each pipeline needs a sub-directory.
- `--refresh <seconds>`: Refresh time used to detect new or updated pipelines and detect if a pipeline should run. Recommended value is 60 seconds or more.

### Pipelines

Each pipeline needs a sub-directory with a `pipeline.json` file together with all script files.

A `pipeline.json` file contains:

```json
{
  "id": "catalog-loader",
  "expression": "30 0,4,8,16,20 * * *",
  "stages": ["download", "import"],
  "jobs": [
    {
      "id": "download-catalog",
      "stage": "download",
      "script": "download-catalog.sh"
    },
    {
      "id": "import-catalog",
      "stage": "import",
      "script": "import-catalog.sh"
    }
  ]
}
```

- `id`: An unique identifier is required for both pipeline and jobs.
- `expression`: CRON-like expression with minutes (0 to 59), hours (0 to 23), days (1 to 31), months (1 to 12) and weekdays (1 for Monday to 7 for Sunday).
- `stages`: A pipeline is separated into stages. This is the execution order for stages. All stage jobs are executed in parallel.
- `stage`: Stage identifier for a job.
- `script`: Script file relative to the pipeline folder.

### States

A `state.json` file is created for each pipeline to save state information.

This file is created automatically and **should never be edited** while the scheduler is still running.

A `state.json` file contains:

```json
{
  "id": "catalog-loader",
  "active": false,
  "timestamp": "2019-07-13T16:00:00.407295085Z"
}
```

- `id`: Unique pipeline identifier.
- `active`: If the pipeline is running.
- `timestamp`: Timestamp in ISO 8601 format with the previous run date.

## Improvements

Although this scheduler works, there are some improvements that could be done:

- Instead of a single service, create a daemon and a client using IPC.
- Use a local database (like SQLite) to store pipeline states.
- Create a Systemd service file.
