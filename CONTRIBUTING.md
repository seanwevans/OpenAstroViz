# Contributing

OpenAstroViz welcomes pull requests from everyone. The typical workflow is:

1. Fork the repository and clone your fork.
2. Create a descriptive feature branch.
3. Commit logically separated changes with clear messages.
4. Push the branch to your fork and open a pull request against `main`.
5. Keep the PR rebased on the latest `main` until it is merged.

CI runs on every commit. Ensure `pre-commit` passes locally before pushing by
running `./scripts/setup.sh` once and then `pre-commit run --files <changed files>`.

### Code style

* **C/C++/CUDA:** formatted with `clang-format`.
* **Rust:** formatted with `rustfmt`.
* **JavaScript/TypeScript:** formatted with `eslint --fix` and `prettier`. Run
  `npm run lint` in `web/` before committing.
* **CUDA kernels:** follow the guidance in [docs/cuda_style.md](docs/cuda_style.md).

PRs that do not pass formatting checks will be blocked by CI.

