# Python PyPI Release Runbook

This project publishes `bd-payment-gateway` (Python package) from
`bd-payment-gateway-py` using a GitHub Actions trusted publishing workflow.

## 1) One-time setup

1. Confirm package metadata in `bd-payment-gateway-py/pyproject.toml` is current.
2. Create GitHub environment `pypi` in repository settings.
3. (Recommended) Require manual approval for the `pypi` environment.
4. In PyPI, configure a Trusted Publisher for this repository with:
   - Owner: your GitHub org/user name
   - Repository: `bd-payment-gateway`
   - Workflow name: `publish-python.yml`
   - Environment name: `pypi`

For a new package that does not yet exist on PyPI, create a pending publisher in
PyPI first with the same values above, then run the first release workflow.

## 2) Before each release

1. Build and validate artifacts locally:
   - `cd bd-payment-gateway-py`
   - `uv build`
   - `uvx twine check dist/*`

## 3) Release from GitHub UI (manual dispatch)

1. Open the repository `Actions` tab.
2. Open workflow `Publish Python Package`.
3. Click `Run workflow`.
4. Fill workflow inputs:
   - `version`: package version (e.g. `0.1.1`), creating tag `py-v0.1.1`
   - `changelog`: markdown release notes body
   - `release_name`: optional (defaults to `Python v<version>`)
   - `target_commitish`: branch to release from (default `main`)
   - `prerelease`: mark release as pre-release
5. Run workflow.
6. Workflow behavior:
   - Dispatch run updates both version files to the input version.
   - It commits and pushes the version bump to `target_commitish`.
   - It creates a GitHub release with your changelog.
   - Release-published event runs wheel/sdist builds.
   - Publish job uploads to PyPI via Trusted Publishing.

Note: If the target branch is protected from direct pushes, either allow this
workflow to push, or release from an unprotected release branch.

## 4) Do I need a PyPI token?

No, not for this workflow. Trusted Publishing does not require a manually
created PyPI API token.

You do need:

- A PyPI account with permission for this package/project.
- Trusted Publisher configuration in PyPI matching this repository/workflow.
- `id-token: write` permission in publish job (already configured).

If you publish manually from local CLI (`uv publish` or `twine upload`), then
you need a PyPI API token.

## 5) Install verification

After release:

- `pip install bd-payment-gateway`
- `uv add bd-payment-gateway` (project dependency)
- `uv pip install bd-payment-gateway` (pip-compatible flow via uv)
