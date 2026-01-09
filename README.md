# ClinLogix

ClinLogix is a Rust-based command-line tool designed for **health informatics and digital health workflows**.  
It provides two main capabilities: **scanning health IT log files** and **validating real FHIR JSON resources** using a FHIR validation service.  
The project also includes **Docker support** for reproducible execution and a **GitHub Pages website** for presentation.

Motivation

Healthcare IT systems generate large volumes of **logs** and **structured clinical data**.  
Manually inspecting these files is **time-consuming**, **error-prone**, and difficult to automate.

ClinLogix was built to:
- **Speed up troubleshooting** of clinical system logs  
- **Validate FHIR resources** before ingestion or exchange  
- **Demonstrate a practical CLI tool** aligned with digital health workflows  

Features

- **Log scanning** with summarized error and warning counts  
- **Error-only output** for rapid triage  
- **JSON output mode** for automation and scripting  
- **FHIR JSON validation** using a `$validate` service  
- **Clear PASS / FAIL results** with detailed issues  
- **Docker support** (no Rust installation required)  

Command-Line Usage

**Scan log files:**

    cargo run -- scan demo-healthit.log
    cargo run -- scan demo-healthit.log --errors-only
    cargo run -- scan demo-healthit.log --json

**Example output:**

    ClinLogix Report
    ----------------
    File: demo-healthit.log
    Total lines: 6
    Errors: 2
    Warnings: 1

FHIR Validation

**Validate a correct FHIR resource:**

    cargo run -- validate examples/patient.json

**Validate an intentionally incorrect resource:**

    cargo run -- validate examples/patient-bad.json

The validation output includes:
- **HTTP status**
- **PASS / FAIL result**
- **Errors and warnings** returned by the validation service

FHIR Validation Details

ClinLogix validates FHIR JSON resources by calling a **public FHIR validation service** using the `$validate` operation.  
Validation responses are parsed from **OperationOutcome** resources.

**Default validation endpoint:**

    https://server.fire.ly

**Override the base URL if needed:**

    cargo run -- validate examples/patient.json --base-url https://example.fhir.server

Run with Docker

ClinLogix can be built and executed using **Docker**, allowing reproducible runs without installing Rust.

**Build the Docker image:**

    docker build -t clinlogix:latest .

**Validate FHIR resources:**

    docker run --rm -v "$PWD:/data" clinlogix:latest validate /data/examples/patient.json
    docker run --rm -v "$PWD:/data" clinlogix:latest validate /data/examples/patient-bad.json

**Scan log files:**

    docker run --rm -v "$PWD:/data" clinlogix:latest scan /data/demo-healthit.log --json

Architecture Diagram

**ClinLogix – High-Level Architecture**

    User (Terminal / Script / CI)
                |
                | CLI Command (scan | validate)
                v
        +----------------------+
        |    ClinLogix CLI     |
        |  (Rust Application) |
        +----------+-----------+
                   |
        +----------+-----------+
        |                      |
        v                      v
+--------------------+   +---------------------------+
| Log File Processor |   | FHIR Validation Client    |
| (scan errors)      |   | ($validate request)       |
+----------+---------+   +------------+--------------+
           |                          |
           |                          | HTTP request
           v                          v
+--------------------+   +---------------------------+
| Summary Formatter  |   | External FHIR Server      |
| (text / JSON)      |   | (e.g. server.fire.ly)    |
+----------+---------+   +------------+--------------+
           |                          |
           +------------+-------------+
                        |
                        v
            +-------------------------------+
            |        CLI Output             |
            | - Error / warning counts      |
            | - JSON summaries              |
            | - PASS / FAIL validation      |
            | - Validation issues           |
            +-------------------------------+

Deployment Note

ClinLogix can be executed:
- **Natively via Rust** (cargo run or release binary)
- **Inside Docker** for environment-independent execution

Project Structure

    clinlogix/
    ├── src/
    │   └── main.rs              CLI implementation (scan + validate)
    ├── examples/
    │   ├── patient.json         Valid FHIR Patient example
    │   └── patient-bad.json     Invalid FHIR Patient example
    ├── docs/
    │   ├── index.html           GitHub Pages website
    │   └── Clinlogix_lean_canvas_doc.pdf
    ├── Dockerfile               Docker build definition
    ├── .dockerignore
    ├── Cargo.toml
    └── README.md

Project Website

**https://princey9.github.io/clinlogix/**

Notes

- **Synthetic data only** is used  
- **No real patient data** is included  
- Public FHIR validation services may occasionally return server-side errors  

Future Improvements

- **Support for additional FHIR resource types**
- **Local validation service** via Docker Compose
- **Structured JSON output** for validation results
- **CI integration** for automated checks

License

**MIT License**

Copyright (c) 2026 Duru Princess Ifeayinwa

Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files to use, copy, modify, and distribute the software for **academic and non-commercial purposes**.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND.

Author

**Duru Princess Ifeayinwa**  
Health Informatics