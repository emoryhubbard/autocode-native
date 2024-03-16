# Autocode Native

Autocode Native is a Rust-based application developed to run natively on Ubuntu. It is designed to improve upon the [Autocode](https://github.com/emoryhubbard/Autocode) web app by handling more complex code prompts, referred to as job descriptions. This project employs task decomposition and a task management algorithm to create incremental, tested changes before moving on to the next step of the job. Currently, the algorithm manages the steps and receives responses for actions it needs to take. The ultimate goal is to allow these job descriptions to be completed. The Future Work section describes the additional work needed before these actions can be automatically implemented.

**[Autocode Native Demo Video](https://youtu.be/Iq5_HaKzL6Y)**

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

- Implement an Extract API endpoint to enable Autocode Native to extract JSX code, based on similar work with [ExtractJS](https://github.com/emoryhubbard/ExtractJS).
- Implement file modification using the ChatGPT responses, using the Extract endpoint.
- Implement automated running and testing of generated code based on job descriptions. Currently the Rust crate chromiumoxide is the top candidate for testing.
- Enhance user experience by adding additional customization options for generated code.

