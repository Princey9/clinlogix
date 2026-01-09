# ClinLogix

ClinLogix is a Rust-based command-line tool designed for health informatics and digital health workflows. It provides two main capabilities: scanning health IT log files for errors and warnings, and validating real FHIR JSON resources using a FHIR validation service. The project also includes Docker support for reproducible execution and a GitHub Pages website for presentation.

Motivation

Healthcare IT systems generate large volumes of logs and structured clinical data. Manually inspecting these files is time-consuming, error-prone, and difficult to automate. ClinLogix was built to speed up troubleshooting of clinical system logs, validate FHIR resources before ingestion or exchange, and demonstrate a practical CLI tool aligned with digital health workflows.

Features

- Scan log files and summarize errors and warnings  
- Error-only output and JSON output modes  
- Validate real FHIR JSON resources using a FHIR validation service  
- Clear PASS / FAIL validation results with detailed issues  
- Docker support (no Rust installation required)  
- Simple CLI interface suitable for automation and scripting  

Command-Line Usage

Scan log files:

    cargo run -- scan demo-healthit.log
    cargo run -- scan demo-healthit.log --errors-only
    cargo run -- scan demo-healthit.log --json

Example output:

    ClinLogix Report
    ----------------
    File: demo-healthit.log
    Total lines: 6
    Errors: 2
    Warnings: 1

Validate FHIR JSON resources:

Validate a correct FHIR resource:

    cargo run -- validate examples/patient.json

Validate an intentionally incorrect resource:

    cargo run -- validate examples/patient-bad.json

The validation output includes the HTTP status, a PASS or FAIL result, and any issues returned by the validator.

FHIR Validation Details

ClinLogix validates FHIR JSON resources by calling a public FHIR validation service using the $validate operation. Validation responses are parsed from OperationOutcome resources and displayed as errors or warnings. The default validation endpoint is:

    https://server.fire.ly

A different validation base URL can be supplied if required:

    cargo run -- validate examples/patient.json --base-url https://example.fhir.server

Run with Docker

ClinLogix can be built and executed using Docker without installing Rust.

Build the Docker image:

    docker build -t clinlogix:latest .

Validate FHIR resources:

    docker run --rm -v "$PWD:/data" clinlogix:latest validate /data/examples/patient.json
    docker run --rm -v "$PWD:/data" clinlogix:latest validate /data/examples/patient-bad.json

Scan log files:

    docker run --rm -v "$PWD:/data" clinlogix:latest scan /data/demo-healthit.log --json

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

    ClinLogix — High-Level Architecture
==================================

                ┌───────────────────────────┐
                │           User             │
                │  (Terminal / Script / CI)  │
                └─────────────┬─────────────┘
                              │
                              │ CLI Command
                              │ (scan | validate)
                              ▼
                ┌───────────────────────────┐
                │        ClinLogix CLI       │
                │      (Rust Application)    │
                └─────────────┬─────────────┘
                              │
        ┌─────────────────────┴─────────────────────┐
        │                                           │
        │                                           │
        ▼                                           ▼
┌───────────────────────┐           ┌───────────────────────────┐
│   Log File Processor  │           │     FHIR Validator Client  │
│  (Error/Warning Scan) │           │   (FHIR $validate call)    │
└─────────────┬─────────┘           └─────────────┬─────────────┘
              │                                   │
              │                                   │ HTTP Request
              │                                   │
              ▼                                   ▼
┌───────────────────────┐           ┌───────────────────────────┐
│  Summary & Formatting │           │   External FHIR Server     │
│ (Text / JSON Output)  │           │   (e.g. server.fire.ly)   │
└─────────────┬─────────┘           └─────────────┬─────────────┘
              │                                   │
              │                                   │ OperationOutcome
              ▼                                   ▼
        ┌─────────────────────────────────────────────────┐
        │               CLI Output (Terminal)              │
        │  - Error / warning counts                        │
        │  - JSON summaries                                │
        │  - PASS / FAIL validation result                 │
        │  - Validation issues (errors / warnings)         │
        └─────────────────────────────────────────────────┘

------------------------------------------------------------

Deployment Option:

ClinLogix can be executed:
• Natively via Rust (cargo run / release binary)
• Inside Docker for reproducible, environment-independent runs


Project Website

The project website is hosted using GitHub Pages and presents the tool, usage examples, and download links:

https://princey9.github.io/clinlogix/

Notes

This project uses only synthetic data. No real patient or clinical data is included. Public FHIR validation services may occasionally return server-side errors, which are outside the control of this tool.

Future Improvements

Potential future improvements include support for additional FHIR resource types, local validation services via Docker Compose, JSON output for validation results, and CI integration for automated checks.

MIT License

Copyright (c) 2026 Duru Princess Ifeayinwa

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, and distribute the Software, for
academic and non-commercial purposes.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.

