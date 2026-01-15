# ClinLogix

ClinLogix is a Rust-based command-line tool for health informatics and digital health workflows. It provides a single unified CLI that supports both health IT log analysis and validation of real FHIR JSON resources using an external FHIR validation service. The project includes Docker-based deployment and a GitHub Pages website for presentation and distribution.

Motivation

Healthcare IT systems generate large volumes of logs and structured clinical data. Manually inspecting these files is time-consuming, error-prone, and difficult to automate. ClinLogix was developed to support faster troubleshooting of clinical systems and to validate FHIR data before exchange or ingestion, reflecting real-world health informatics practice.

Features

- Log scanning with summarized error and warning counts  
- Error-only output for rapid troubleshooting  
- JSON output mode for automation and scripting  
- FHIR JSON validation using a FHIR $validate service  
- Clear PASS / FAIL results with detailed validation issues  
- Docker-based deployment for reproducible execution  

Testing Release Binaries

The Linux release binaries are statically linked and intended for Linux systems.
Attempting to run the Linux binary on macOS will result in:
zsh: exec format error

Reproducible Checks

Run the verification script and save logs to a timestamped folder under `out/`:

    bash scripts/run_checks.sh

Optional overrides:

    BASE_URL=https://server.fire.ly bash scripts/run_checks.sh
    RUN_BATCH=1 bash scripts/run_checks.sh


Command-Line Usage

Scan health IT log files:

    cargo run -- scan demo-healthit.log
    cargo run -- scan demo-healthit.log --errors-only
    cargo run -- scan demo-healthit.log --json

Validate FHIR JSON resources:

    cargo run -- validate examples/patient.json
    cargo run -- validate examples/patient-bad.json

FHIR Validation

ClinLogix validates FHIR JSON resources by calling a remote FHIR $validate endpoint. Validation responses are parsed from OperationOutcome resources and presented clearly in the terminal.

Default validation service:

    https://server.fire.ly

Validating Synthea Bundles

Synthea bundles often rely on US Core and other implementation guide profiles. When you validate them against the default Firely server, the server may not have those packages installed, and it will return errors such as "Unable to resolve reference to profile ...". ClinLogix now highlights these issues under a dedicated theme while still grouping categories by severity, code, and message and reporting JSON path expressions with best-effort line numbers.

Generate Synthea data (optional):

- Install Synthea from https://github.com/synthetichealth/synthea
- Run a small generation, for example: `./run_synthea -p 1`
- Copy a FHIR bundle JSON into `synthea/` (or `examples/`) and run:

    for f in synthea/*.json; do
        cargo run -- validate "$f" --base-url https://server.fire.ly
    done

If you run into profile resolution errors, try alternative validation endpoints (options, not requirements):

- Firely server (default): `https://server.fire.ly`
- HAPI public R4 server: `https://hapi.fhir.org/baseR4`
- A local validator setup (for example, a HAPI instance or another validator that has US Core packages installed)

Tips for large bundles:

- Pipe output to a pager: `for f in synthea/*.json; do clinlogix validate "$f"; done | less -R`
- Jump to summaries with ripgrep: `for f in synthea/*.json; do clinlogix validate "$f"; done | rg -n "Themes:|Top Issue Groups|Validate:"`

Docker Deployment and Execution

ClinLogix can be built and executed entirely using Docker, without requiring Rust to be installed on the host system. This enables reproducible execution across different environments.

Build the Docker image:

    docker build -t clinlogix:latest .

Run FHIR validation:

    docker run --rm -v "$PWD:/data" clinlogix:latest validate /data/examples/patient.json
    docker run --rm -v "$PWD:/data" clinlogix:latest validate /data/examples/patient-bad.json

Run log scanning:

    docker run --rm -v "$PWD:/data" clinlogix:latest scan /data/demo-healthit.log --json

Install (macOS)

After building, install ClinLogix as a system-wide CLI:

    sudo cp target/release/clinlogix /usr/local/bin/clinlogix
    clinlogix --help

Architecture Overview

[View Architecture Diagram](docs/clinlogix_architecture.png)

Deployment Summary

ClinLogix is deployed as a native Rust CLI application, as a Docker container for environment-independent execution, and as a static project website hosted on GitHub Pages.

Project Structure

    clinlogix/
    ├── src/
    │   └── main.rs
    ├── examples/
    │   ├── patient.json
    │   └── patient-bad.json
    ├── docs/
    │   ├── index.html
    │   └── Clinlogix_lean_canvas_doc.pdf
    ├── Dockerfile
    ├── .dockerignore
    ├── Cargo.toml
    └── README.md

Project Website

https://princey9.github.io/clinlogix/
Website source: docs/index.html

Enable GitHub Pages and set the About link:

1. GitHub repo → Settings → Pages
2. Source: Deploy from a branch
3. Branch: main (or default) + folder: /docs
4. Save → copy the generated URL
5. Repo main page → About → Website → paste the Pages URL

Repository Settings (GitHub About)

To update the GitHub About section manually:

1. Go to the GitHub repo and find the About box in the right sidebar.
2. Click the edit (pencil) icon.
3. Add the GitHub Pages URL, a short description, and relevant topics (for example: fhir, rust, cli, health-it).

To enable code review requirements:

1. Go to Settings → Branches → Add branch protection rule.
2. Target the default branch, then enable “Require a pull request before merging.”
3. Set the required number of approvals and save the rule.

Notes

Only synthetic data is used. No real patient data is included. External FHIR validation services may occasionally be unavailable.

License

MIT License

Copyright (c) 2026 Duru Princess Ifeayinwa

Permission is hereby granted to use, copy, modify, and distribute this software for academic and non-commercial purposes.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND.

Author

Duru Princess Ifeayinwa  
Health Informatics
