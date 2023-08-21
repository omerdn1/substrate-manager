### 🚧 **Note: Substrate Manager is currently in active development. While we strive to ensure stability, please consider backing up your projects before use. & Stay tuned for regular feature updates!**

# Substrate Manager CLI

Streamline Your [Substrate](https://substrate.io/)-based Blockchain Development.

Welcome to Substrate Manager, your interactive CLI toolkit for streamlined development of Substrate-based blockchains and smart contracts. Whether you're building isolated blockchains, parachains, custom pallets, or smart contracts, Substrate Manager provides an intuitive interface to manage your projects with ease.

Substrate Manager is written in Rust and takes inspiration from projects like [Cargo](https://github.com/rust-lang/cargo) and [Near-CLI](https://github.com/near/near-cli-rs).

<video src='https://github.com/omerdn1/substrate-manager/assets/10265367/50ac8c97-ba83-4376-b964-bff9babcc5e5' loop autoplay>
</video>

## Approach and Philosophy

Diving into Substrate development can feel like navigating a complex maze, where you'll find yourself having to juggle a bunch of tools, commands, resources and terminologies, leading to a time-consuming and challenging initiation process. Learning and mastering these intricacies can be a significant barrier, often deterring potential developers.

Substrate Manager emerges as a transformative solution to this. It's designed to provide a streamlined experience for all Substrate-based development workflows, effectively breaking down these barriers and aligning with key principles and best practices in the ecosystem. It embodies the following core aspects:

- **Simplified Development:** Substrate Manager takes care of the technical nitty-gritty – setting up libraries, tools, and dependencies. No more grappling with overwhelming guides or deciphering unfamiliar jargon. Start your Substrate journey with confidence and focus on what you do best: building.
- **Adaptive:** Got your own project setup? No worries. Substrate Manager is like a trusty sidekick that seamlessly fits in. It's equipped to understand your project's structure and adapts effortlessly; just run `substrate-manager`!
- **Staying Current:** In a world of rapid tech evolution, Substrate Manager stays in the loop. We keep tabs on the latest trends and best practices in the Substrate ecosystem, ensuring that Substrate Manager remains up-to-date.

## Installation

To install Substrate Manager, ensure you have Rust and Cargo installed on your machine. If not, follow the official [Rust installation guide](https://www.rust-lang.org/tools/install).
Then run:

```sh
cargo install substrate-manager
```

## Usage

Substrate Manager is an interactive CLI tool.
Simply run:

```sh
substrate-manager
```

## Substrate-based chain development

Substrate Manager simplifies the creation and management of all types of Substrate-based chains, including isolated blockchains. It streamlines your workflow with the following features:

- **Project Creation:** Generate new chain projects using templates such as Substrate Node Template, Cumulus & Frontier, or custom templates adhering to Substrate Library Extension (SLE) standards.
- **Launch Your Node:** Launch your chain nodes using your existing `chain_spec` commands, ensuring consistent behavior across projects.
- **Pallet Integration:** Install pallets directly to your runtime from a variety of sources, such as crates.io, Git repositories, local paths, or custom registries, to enhance your chain's functionality.
- **Frontend Interfaces:** Launch Parity's frontend chain interface or your custom frontend to easily interact with your chain.
- **Comprehensive Testing:** Validate your chain's functionality and robustness with comprehensive testing.

## Substrate-based smart contract development

Substrate Manager facilitates the creation, management and deployment of smart contracts on Substrate-based chains. Develop and deploy smart contracts with ease using these capabilities:

- **Project Creation:** Create smart contract projects using [cargo-contract](https://github.com/paritytech/cargo-contract), with future template support like Flipper, OpenZeppelin, etc.
- **Build and Deploy:** Compile and deploy smart contracts using Substrate Smart Contract UI integration.
- **Test Reliability:** Execute tests to ensure the reliability and functionality of your smart contracts.

## Documentation

🏗️ Under construction...

## Upcoming Features and Feature Requests

We are continuously working to enhance Substrate Manager's capabilities. Keep an eye out for upcoming features, and if there's something you'd like to see in the toolkit, don't hesitate to create a feature request in our [GitHub Issues](https://github.com/omerdn1/substrate-manager/issues). Your input drives the evolution of Substrate Manager! ✨

## Get Involved

We welcome contributions from the community. If you find a bug, have an enhancement suggestion, or want to participate in development, check out our [Contribution Guidelines](https://github.com/omerdn1/substrate-manager/blob/main/CONTRIBUTING.md).
Encountered an issue or need help? Create an issue in our [GitHub Issues](https://github.com/omerdn1/substrate-manager/issues).

Let's make Substrate development faster and more enjoyable together! 🚀
