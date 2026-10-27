# goatmtn workspace

1. Open `goatmtn.code-workspace` in VS Code and accept the recommended extensions.
2. Install: rustup (MSVC toolchain + VS Build Tools), .NET SDK, Python 3.11+.
3. `python -m venv sim/.venv` then `sim/.venv/Scripts/pip install -r sim/requirements.txt`
4. Set the GameManaged path (see plugin/README.md).
5. Run the "check all" task.
6. `git init` and commit before the first Claude Code session.
