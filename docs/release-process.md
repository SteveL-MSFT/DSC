# Release Process for DSC

This document describes the release process for DSC starting from GitHub to MSAzure.

The examples in this document are based on DSC 3.3.
The version you need to use will be different, so please be sure to use the correct version in your commands.

## Git Configuration

In the examples, `origin` points to your fork of the DSC repository, and `upstream` points to the main DSC repository.
`msazure` points to the mirror of the DSC repository in MSAzure.

Example:

```powershell
git remote -v

msazure	https://dev.azure.com/msazure/One/_git/DSC (fetch)
msazure	https://dev.azure.com/msazure/One/_git/DSC (push)
origin	git@github.com:SteveL-MSFT/DSC.git (fetch)
origin	git@github.com:SteveL-MSFT/DSC.git (push)
upstream	https://github.com/PowerShell/DSC.git (fetch)
upstream	https://github.com/PowerShell/DSC.git (push)
```

## GitHub preparation

First, check if there are any PRs that need to be backported to the release branch.
This [example query](https://github.com/PowerShell/DSC/pulls?q=is%3Apr+is%3Aclosed+label%3ABackport-Needed+milestone%3A3.3-Approved) checks for the `Backport-Needed` label and the milestone `3.3-Approved`:

If there are any completed PRs that require backport, first make sure your local repository is up to date with the upstream repository:

```powershell
git fetch --all
```

Then checkout the release branch from GitHub:

```powershell
git checkout upstream/release/3.3
```

Create a working branch for the backport:

```powershell
git checkout -b backport-3.3
```

Cherry-pick the PR commits that need to be backported.
PRs should be squash-merged, so there should only be one commit to cherry-pick available at the bottom of the PR page.
If there are multiple PRs that need to be backported, repeat the cherry-pick command for each commit.

```powershell
git cherry-pick <commit-hash>
```

Push the backport branch to your fork:

```powershell
git push origin backport-3.3
```

Create the PR from your fork ensuring the base branch is the release branch (`release/3.3` in this example) as it will default to `main` otherwise.

Address any potential merge conflicts, and ensure the code builds and passes all tests.

## MSAzure preparation

Once the release branch on GitHub is ready, you now need to create a PR to update the release branch in MSAzure.

First, checkout the release branch from GitHub:

```powershell
git fetch --all
git checkout upstream/release/3.3
```

Now push the updated branch to MSAzure:

```powershell
git push msazure HEAD:refs/heads/backport-3.3
```

In the [MSAzure portal](https://msazure.visualstudio.com/One/_git/DSC/pullrequests?_a=mine), create a PR to merge the `backport-3.3` branch into the `release/v3.3` branch.
Ensure the target branch is the release branch (`release/v3.3` in this example) as it will default to `main` otherwise.

Be sure to remove any automatically added work items from the PR description, as they are not relevant to the release process and likely incorrect.
Complete the mandatory compliance checks in the PR and continue when PR is completed.

You may need to manually fix any merge conflicts in the PR.
This needs to be done locally:

```powershell
git checkout msazure/backport-3.3
git pull msazure release/v3.3
# fix any merge conflicts
# build and test to ensure everything is working
./build.ps1 -Clippy
./build.ps1
./build.ps1 -Test
git push msazure backport-3.3
```

## Pipelines

There are two different pipelines that need to be run for the release process, one for the GitHub repository and one for publishing to Packages.Microsoft.com (PMC).

### GitHub Pipeline

The `DSC-OfficialRelease` pipeline is used to create the release artifacts and draft the release on GitHub.
Make sure to mark it as `favorite` so you can find it easily in the future.

When running this pipeline:

- ensure the correct release branch is selected, `release/3.3` in this example
- be sure to check the `OfficialBuild` checkbox and don't check the `PublishToStore` checkbox which currently does not work.

This pipeline will take a while to run and will pause at two points for manual intervention:

- After the build and packaging completes, there is a manual verification step where you can inspect the artifacts and ensure they are correct before continuing.
  Click the `Review` button to continue the pipeline.
- There is an additional `Approval` step before the release is published to GitHub.
  In the `Approval` stage, there is a URL that you need _someone else on the team_ to click to approve the release.
  There are two different approvals, one for `Release` and one for `Deployment`.  Two different people will be needed to approve the release before it can be published to GitHub.

### PMC Pipeline

The `DSC-Official-PMC` pipeline is used to publish the release artifacts to Packages.Microsoft.com (PMC).
Make sure to mark it as `favorite` so you can find it easily in the future.

When running this pipeline:

- ensure the correct release branch is selected, `release/v3.3` in this example
- be sure to check the `Publish PMC Packages` checkbox, otherwise it will only stage the artifacts and not publish them to PMC.

This pipeline will wait for approval before publishing the artifacts to PMC.

> **NOTE**
> You need _someone else on the team with a SAW using their SAW account_ to approve the PMC release
> otherwise, the approval will succeed, but the pipeline will fail during compliance checks and the pipeline will need to be re-run (you can't just re-run the failed stage)

## GitHub Release

Once the GitHub pipeline has completed, the release will be drafted on GitHub.
On the [GitHub Releases page](https://github.com/PowerShell/DSC/releases), you should see a draft release for the version you just released:

- Click the `Edit` button
- Click the `Generate release notes` button to generate the release notes for the release
- Review the content and make any changes you feel are necessary
- Verify the `pre-release` or `latest` release checkboxes are set correctly for the release
- Click the `Publish release` button to publish the release on GitHub

## Store Release

Until the `PublishToStore` checkbox is fixed in the `DSC-OfficialRelease` pipeline, the store release will need to be done manually:

- From the GitHub release page, download the `msixbundle` file
- Go to the [Microsoft Store Partner Center](https://partner.microsoft.com/dashboard) and sign in with your account
- Find the `Desired State Configuration` or `Desired State Configuration (Preview)` app in your dashboard and click on it
- Click on the `Create new submission` button
- Click on the `Packages` tab and upload the `msixbundle` file you downloaded from GitHub
- Save the submission and click on the `Submit to the Store` button to submit the release for certification
