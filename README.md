# Autocode Client

The Autocode Client is the Node successor to the Autocode Native application. Autocode Native was refactored from Rust to TypeScript to allow for more rapid prototyping in this stage of development.

# Autocode Native

Autocode Native is a Rust-based application developed to run natively on Ubuntu. It is designed to improve upon the [Autocode Classic](https://github.com/emoryhubbard/Autocode?tab=readme-ov-file#autocode-classic) web app by helping React developers automate adding features to their apps, with the goal of leveraging task decomposition and a task management algorithm to create incremental, tested changes before moving on to the next step of the job.

**[First Autocode Native Demo Video](https://youtu.be/eV3pdysg3aE)** (see the [Autocode repo](https://github.com/emoryhubbard/Autocode) for the current status of the project)

# Development Environment

Autocode Native is developed using the following technologies and tools:

- **Rust:** Autocode Native is primarily developed using Rust programming language, providing high performance and memory safety.
- **Ubuntu:** The application was developed with and designed to run on the current LTS version of Ubuntu (Jammy Jellyfish).
- **ChatGPT API:** Autocode Native utilizes the ChatGPT API for prompt-based code generation and interaction. An API key is required for accessing ChatGPT services.
- **serde_json:** The serde_json crate is used for parsing and generating JSON data.
- **dotenvy:** Autocode Native uses the actively-maintained dotenvy fork of dotenv for loading environment variables from a `.env` file during development.
- **tokio:** Asynchronous runtime for Rust, used for asynchronous task management.
- **reqwest:** The reqwest crate is used for making HTTP requests to external APIs.
- **Visual Studio Code (VS Code):** The development environment is Visual Studio Code, a powerful and extensible code editor.

# Useful Rust Libraries

- [serde_json](https://crates.io/crates/serde_json)
- [dotenvy](https://crates.io/crates/dotenvy)
- [tokio](https://crates.io/crates/tokio)
- [reqwest](https://crates.io/crates/reqwest)

# Future Work

Note: Some of this work has already been completed. See the [Autocode repo](https://github.com/emoryhubbard/Autocode) for the current status of the project.

- Implement an Extract API endpoint to enable Autocode Native to extract JSX code, based on similar work with [ExtractJS](https://github.com/emoryhubbard/ExtractJS).
- Implement file modification using the ChatGPT responses, using the Extract endpoint.
- Implement automated running and testing of generated code based on job descriptions. Currently the Rust crate chromiumoxide is the top candidate for testing.
- Enhance user experience by adding additional customization options for generated code.

