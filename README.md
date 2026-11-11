# bitbison-github-demo

How to use [Bitbison's GitHub action](https://github.com/bitbison-labs/github)
to review dependency changes and gate builds, with Claude for what needs
judgment. This repository is a working example: every step below is live
here, with a pull request you can open and read.

- **[Tutorial](#tutorial)**: set it up in your repository, one step at a time.
- **[Quick reference](#quick-reference)**: the workflows, the options, what
  each result means, and the demo pull requests.
- **[Details and background](#details-and-background)**: how each review
  decides, and why it is built this way.

The code under review is a small application kept only to give the action
something real to read: a link shortener with a Rust API (`api/`), a web UI
built with vite (`web/`), a Python worker (`worker/`) and a Go CLI (`cli/`),
so one repository has four lockfiles and four build jobs.

## Tutorial

The action has three parts, and you can stop after any of them:

1. the **dependency review** of pull requests and pushes (steps 1 to 5);
2. the **build integration**, which captures what a build job does (step 6);
3. the **gate**, which reports on a captured build and can stop what
   depends on it (step 7).

### 1. Add the secrets

You need a Bitbison organization and a Claude credential. In your Bitbison
user portal, have an administrator create a CI token for the reviewer with
the `mcp/ci/runtime` capability. Then, in your repository's **Settings →
Secrets and variables → Actions** (or your GitHub organization's), add:

| Secret | Value |
|---|---|
| `BITBISON_CI_TOKEN` | the reviewer's CI token |
| `ANTHROPIC_API_KEY` | a Claude API key (or `CLAUDE_CODE_OAUTH_TOKEN`) |
| `BITBISON_INSTANCE` | your Bitbison instance URL |
| `BITBISON_ORGANIZATION` | your organization id, from the user portal |

### 2. Add the review workflow

Create `.github/workflows/bitbison-review.yml` and merge it to your default
branch:

```yaml
name: Bitbison review

on:
  pull_request:
  push:
    branches: ['**']
    tags: ['**']
  delete:
  workflow_dispatch:

jobs:
  review:
    permissions:
      contents: read
      pull-requests: write
      id-token: write
    uses: bitbison-labs/github/.github/workflows/review.yml@master
    secrets: inherit
```

`pull_request` is the review itself. The other triggers let Bitbison record
what each branch and tag pins, and review pushes to the default branch
(step 5); a first setup can start with `pull_request` alone.

**What you see:** a **Bitbison review** run for the merge. Its job summary
has one line, what the branch pins, and its log says the branch was
recorded for the first time: that state is the branch's baseline, so there
is nothing to review yet.

### 3. Open a pull request that changes a dependency

Add or upgrade a direct dependency the way you always do, commit the
manifest and the lockfile, and open a pull request from a branch of the
same repository. Nothing about how you work changes.

**What you see:** one comment from `github-actions[bot]`, kept up to date
as you push. It lists each direct dependency the pull request adds or
upgrades and where it stands with your organization:

> **✅ Every dependency change is covered.** This pull request adds or
> upgrades 1 direct dependency, covered by a grant; nothing is left for a
> person.
>
> | | Release | Change | Where it stands |
> |---|---|---|---|
> | ✅ | `npm/vite@6.4.6` | upgraded from 6.4.3 | Already granted |

A **grant** means your organization allows that release. Bitbison settles
most changes by itself, with no model: a release already granted, one your
rules grant, or one whose analysis in the registry is clean.

See it here:

| Pull request | What it changes | The comment says |
|---|---|---|
| [#6](../../pull/6) | adds a package new to the organization | **Granted**: the review's own request was granted |
| [#2](../../pull/2) | upgrades a package | `upgraded from 6.4.3`, **Already granted** |
| [#5](../../pull/5) | adds two packages the organization uses elsewhere | **Already granted** |
| [#3](../../pull/3) | upgrades two packages, the lockfile alone | `upgraded from …`, **Already granted** |
| [#7](../../pull/7) | removes a direct dependency | **Removed**, nothing to decide |
| [#4](../../pull/4) | changes no dependency | no comment at all |
| [#1](../../pull/1) | adds eight dependencies in cargo, npm, PyPI and Go at once | one table for all four lockfiles |

### 4. Read a review that needs you

A few changes are not settled by a grant. The comment's first line then
counts them (**⏳ 1 of 1 dependency change needs a person**), and a section
**What a person must still decide** links each to its page in the Bitbison
console. Claude is pulled in for these, and comments inline on the manifest
line with what it found and the way out.

- **A release the registry reviewed and found a problem in.** The request is
  refused as `review`. Claude names the way out, usually a fixed version or
  an alternative your organization already uses; only a person can override
  the review. See [#9](../../pull/9): `rustpython-vm 0.5.0` copies its
  build environment's credentials into what it compiles, and Claude
  suggests `0.6.0`, which the organization already uses.
- **Another repository's code.** A submodule added, moved, pointed elsewhere
  or removed is listed with a link to what changed. No grant covers a
  commit of someone else's repository, so a person reviews it. See
  [#8](../../pull/8).
- **A known vulnerability.** Every advisory on the direct changes is in an
  **Advisories** table, linked, with the version that fixes it. To have
  Claude also judge whether each one reaches your code, add to the review
  job:

  ```yaml
      with:
        vulnerabilities: high   # off (the default) | critical | high | medium | low
  ```

  Claude searches your code and answers **affected**, **not affected** or
  **unsure** for each advisory at that severity or above, with the way out.
  See [#11](../../pull/11): the code compiles a caller's pattern with
  `regex 1.5.4`, so its denial-of-service advisory is **affected**, fixed
  by a lockfile-only upgrade. In [#10](../../pull/10) the advisory of
  `atty 0.2.14` needs Windows and a custom allocator: **not affected**.

Claude's comment sits on the manifest line that brought the release in,
and where one line settles it, the comment carries a suggested change you
can commit from the pull request. From [#9](../../pull/9), on
`api/Cargo.toml`:

> **`rustpython-vm 0.5.0`: not covered. The registry has reviewed it and
> found it exposes secrets.**
>
> **What the registry found:** The registry's review covers rustpython-vm
> 0.3.0 to 0.5.0. Each of these versions builds its compile environment
> into every program that links it. That review stands for the
> organization, so I did not audit this release. Only a person can override
> it.
>
> **Way out:** Use `cargo/rustpython-vm@0.6.0`. It is already covered by
> the organization's grant (analysis clean, risk none, 0 findings), and it
> is the version this organization already uses in another repository.
>
> ```suggestion
> rustpython-vm = "0.6.0"
> ```

And from [#11](../../pull/11), where the fix needs no manifest change:

> **CVE-2022-24713 / GHSA-m5pq-gvj9-9vr8 (high): regex denial of service.
> Affected.**
>
> `GET /links?match=<regex>` passes the caller's query string straight to
> `regex::Regex::new(&pattern)` (`api/src/main.rs:49`), so any client can
> tie up the server with one request.
>
> **Fix:** 1.5.5 fixes it, and the requirement `"1.5.4"` already allows it.
> Only the lockfile needs to change, with no code changes:
> `cargo update -p regex --precise 1.5.5`

By default, the dependency review job only reports (`fail-on: never`), so
nothing here blocks the merge. With any `fail-on` setting, it reports one
of these three states in the summary comment and the job annotation:

| State | What it means | Job annotation |
|---|---|---|
| 🟥 Denied | your organization has said no to at least one change: a `deny` rule, the built-in rule for a release reviewed or reported as malicious, an approved deny audit, or a revoked grant | error |
| 🟨 Needs a person | no change is denied, but at least one has no grant yet and is waiting on a person | warning |
| 🟩 Covered | every change has a grant | notice |

An error annotation does not by itself fail the job. `fail-on` controls
when the job fails, as shown below. To block merges, choose `denied` or
`uncovered` and make the review job a required check:

| `fail-on` | 🟥 Denied | 🟨 Needs a person | 🟩 Covered |
|---|---|---|---|
| `never` (the default) | passes | passes | passes |
| `denied` | **fails** | passes | passes |
| `uncovered` | **fails** | **fails** | passes |

To treat a review conclusion or an advisory as a reason to deny a change,
add a `deny` rule in Bitbison. `fail-on` still controls whether that denial
fails the job.

### 5. Merge, and let the push be reviewed

With `push` in the workflow's triggers (step 2), a push to the default
branch is reviewed too. There is no pull request to comment on, so the
review is on the run's **job summary**, and the job's annotation says
whether a person has something to decide.

A push is reviewed by **coverage**: what it brought since the last state
Bitbison recorded, and every direct dependency the branch pins that no
grant covers and no one in the project has asked for. A push that changes
no dependency says nothing.

Every other push, every tag and every deletion records what that ref pins,
with no review. Each grant in the Bitbison console then shows where its
release is in use, since when, and when it stopped.

See it here, on the job summary of each run: the pushes that merged
[#1](../../actions/runs/42774622516), [#2](../../actions/runs/42775246983)
and [#3](../../actions/runs/42775311875) list what each brought, all
covered. The [first push](../../actions/runs/42774310833), which added the
workflow, only recorded the branch: its baseline.

### 6. Capture your build

Set up the build integration from your Bitbison user portal. It gives the
repository the secrets `BITBISON_CHANNEL`, `BITBISON_TOKEN` and
`BITBISON_INSTANCE`. Add its step to each job that builds, straight after
the checkout:

```yaml
    steps:
      - uses: actions/checkout@<commit>
      - uses: bitbison-labs/github@<commit>
        with:
          channel: ${{ secrets.BITBISON_CHANNEL }}
          token: ${{ secrets.BITBISON_TOKEN }}
          instance: ${{ secrets.BITBISON_INSTANCE }}
      # ... your build
```

The step starts a capture of the whole job (every process, file and
connection), and seals and submits it when the job ends. The jobs of one
workflow run are one Bitbison build. Nothing is reported in GitHub yet:
that is the gate's work.

See it here: [`bitbison.yml`](.github/workflows/bitbison.yml) has four
captured jobs, `api`, `web`, `worker` and `cli`. In a repository with
several components, have each job check out its own
(`sparse-checkout: api`), so a job's capture and its dependencies are that
component's alone.

### 7. Gate what depends on the build

Add the gate as a job between your build jobs and whatever ships them, with
`needs:` on both sides:

```yaml
  bitbison:
    needs: [api, web, worker, cli]      # your build jobs
    permissions:
      contents: read
      pull-requests: read
      checks: write
      id-token: write
    uses: bitbison-labs/github/.github/workflows/gate.yml@master
    with:
      gate: report
    secrets: inherit

  publish:
    needs: [api, web, worker, cli, bitbison]
    # ... runs only when the gate passes
```

**What you see:** a check on the commit, **Bitbison build report**, in the
pull request's Checks tab and on the gate job's summary. It leads with the
gate's decision, the run's verdict and each job's beside the default
branch's, then the findings. What the build did besides is folded under
them: every host it connected to, what it produced, its dependencies in
numbers, and every package it resolved that an advisory names. A clean
build is a dozen lines.

`gate: report` never fails the job. When you are ready, `block` fails a
build that is not clean, and `agent` lets Claude decide whether what was
flagged is a real risk. Claude never runs on a clean build.

See it here, with `gate: agent`:

| Pull request | The build | The gate |
|---|---|---|
| [#12](../../pull/12) | nothing new | clean; passes without Claude, publish runs |
| [#13](../../pull/13) | a build script looks up the latest Rust version | flagged; Claude finds no real risk; passes, publish runs |
| [#14](../../pull/14) | a build script compiles an unpinned, unverified download into the API | flagged; Claude finds a real risk; fails, publish is skipped |

When a build is flagged, Claude's sentence on it follows the jobs, and
under the findings its verdict and next step for each; its analysis in full
is folded below. From [#14](../../pull/14)'s check:

> **Gate (`agent`): failed.** Claude found a real risk.
>
> **⚠️ Flagged.** Run `42775389107-1`: 3 open findings in 4 jobs, against
> `master`'s run `42776385308-1`.
>
> PR #14 adds an `api/build.rs` that uses curl to download LICENSE-MIT from
> the `main` branch of rust-lang/rust. Nothing pins or checks the download,
> and `include_str!` builds it into the shipped corral-api binary. All
> three open findings are this one download, and each is a real risk.
>
> | Finding | Real risk | Why | Next step |
> |---|---|---|---|
> | `egress` `corral-api@0.1.0` | **yes** | … The URL points at a moving branch, nothing checks a digest, and the build compiles the result. | Remove the download from build.rs. Commit the license text to the repo (for example `api/LICENSE-MIT`) and use `include_str!("../LICENSE-MIT")`. … |

## Quick reference

### The three parts

| Part | You add | It runs on | It reports |
|---|---|---|---|
| Dependency review | `bitbison-labs/github/.github/workflows/review.yml`, as a job | `pull_request`; `push`, tags, `delete`, `workflow_dispatch` | one comment on the pull request; the job summary on a push |
| Build integration | `bitbison-labs/github`, as a step after the checkout | every job you build in | nothing in GitHub: it captures the job |
| Gate | `bitbison-labs/github/.github/workflows/gate.yml`, as a job that `needs` the builds | whatever runs your build workflow | the **Bitbison build report** check and the job summary; its job fails when the gate does |

### Secrets

| Secret | Needed by | From |
|---|---|---|
| `BITBISON_CI_TOKEN` | review, gate | a CI token with `mcp/ci/runtime` (or `mcp/ci`), created by an administrator |
| `ANTHROPIC_API_KEY` or `CLAUDE_CODE_OAUTH_TOKEN` | review, gate | your Claude account; neither with workload identity federation |
| `BITBISON_INSTANCE` | all three | your instance URL |
| `BITBISON_ORGANIZATION` | review, gate | your organization id; not needed when `BITBISON_CHANNEL` is set |
| `BITBISON_CHANNEL`, `BITBISON_TOKEN` | build integration | the build integration's installation guide |

### Permissions

| Job | `permissions:` |
|---|---|
| Review | `contents: read`, `pull-requests: write`, `id-token: write` |
| Gate | `contents: read`, `pull-requests: read`, `checks: write`, `id-token: write` |
| A build job | `contents: read` |

### What the dependency review says

| It says | What happened | Claude | You |
|---|---|---|---|
| *(no comment)* | no direct dependency or submodule changed | no | nothing |
| ✅ **Already granted** | a grant was in force before the change | no | nothing |
| ✅ **Granted** / **Granted by rule** | the review's request, or one of your rules, granted it now | no | nothing |
| ✅ **Granted by audit** | Claude audited a release the registry has not reviewed, below your risk floor | yes | nothing |
| ⏳ **Waiting on a person** | an audit at or above your risk floor | yes | approve or reject, linked |
| ⏳ **Not covered**: refused as `review` | the registry reviewed the release and found a problem | yes | take the way out Claude names, or override, linked |
| ⏳ **Denied by a rule** | one of your rules refuses the release | yes | another release or package |
| **Removed** | a direct dependency is gone | no | nothing |
| **Advisories** table | every advisory on the direct changes, linked | from `vulnerabilities:` up | upgrade where one reaches you |
| Submodule table | a submodule added, moved, repointed or removed | no | review the other repository's change |

### Review options

Under `with:` on the review job:

| Option | Default | Meaning |
|---|---|---|
| `vulnerabilities` | `off` | `critical`, `high`, `medium` or `low`: the least severity at which Claude judges whether an advisory reaches your code. An unrated advisory counts as `low` |
| `dependency-grants` | `true` | `false`: the review only reads, and requests nothing |
| `fail-on` | `never` | `never`: only reports; `denied`: fails only on a denied change; `uncovered`: fails on any change without a grant |
| `track` | `all` | what is recorded besides the default branch: `branches`, `tags`, both, or `default-branch` for it alone |
| `model` | the action's | the Claude model |
| `runs-on` | `"ubuntu-latest"` | the runner, as JSON |

A failed review job blocks a merge only when it is a required check.
`fail-on-uncovered: true` is the old spelling of `fail-on: uncovered`.

### Gate options

| `gate` | A clean build | A build that is not clean |
|---|---|---|
| `report` (the default) | passes | passes, with the report |
| `block` | passes | fails, until a person accepts its findings with a policy or the change removes them |
| `agent` | passes | Claude decides: a real risk fails, none passes, with its reason for each finding |
| `off` | does nothing | does nothing |

| Option | Default | Meaning |
|---|---|---|
| `analysis` | `true` | Claude explains a build that is not clean. `false` keeps the report deterministic; `gate: agent` needs it |
| `wait-minutes` | `15` | how long to wait for the build's analysis |
| `on-timeout` | `pass` for `report`, else `fail` | what a late analysis does |

### To change how much is automatic

| To | Do |
|---|---|
| Have Claude read every change and request nothing | `dependency-grants: false` |
| Block a merge only when your organization has denied a change | `fail-on: denied`, and make the review job a required check |
| Block a merge until every change has a grant | `fail-on: uncovered`, and make the review job a required check |
| Decide which of Claude's audits wait for a person | an administrator runs `bitbison settings --audit-person-from-risk none\|low\|medium\|high\|critical` (`high` by default) |
| Keep your audits from being proposed to the registry | `bitbison settings --share-audits false` |
| Adopt what a branch pinned before Bitbison first saw it | a person runs `bitbison grant baseline` on the default branch |
| Never let a flagged build through without a person | `gate: block` |
| Let Claude clear false positives on its own | `gate: agent` |
| Never pull in Claude for builds | `analysis: false`, with `report` or `block` |
| Stop a build finding from coming back | a `false_positive` policy for the repository, or an `accepted_download` for a package's download, in the Bitbison console |

### The demo pull requests

| Recipe | Pull request | Result |
|---|---|---|
| Four ecosystems at once | [#1](../../pull/1) (merged) | 8 dependencies covered; advisories judged inline; a submodule listed |
| No dependency changes | [#4](../../pull/4) | no comment |
| Packages already trusted | [#5](../../pull/5) | Already granted |
| A new package | [#6](../../pull/6) | Granted |
| An upgrade that fixes advisories | [#2](../../pull/2) (merged) | upgraded, Already granted |
| Routine upgrades | [#3](../../pull/3) (merged) | upgraded, Already granted |
| A dependency removed | [#7](../../pull/7) | Removed |
| A submodule moved | [#8](../../pull/8) | listed for a person |
| Supply chain risk | [#9](../../pull/9) | refused as `review`; a fixed version suggested |
| A vulnerability the code never reaches | [#10](../../pull/10) | not affected |
| A vulnerability the code reaches | [#11](../../pull/11) | affected, with the way out |
| Clean build | [#12](../../pull/12) | clean, gate passes without Claude, publish runs |
| Build false positive | [#13](../../pull/13) | Claude clears it, publish runs |
| Build real risk | [#14](../../pull/14) | gate fails, publish skipped |

## Details and background

### The idea: autonomy first, people for what needs them

The goal is to settle as much as possible without a person and without
blocking anyone:

- **Bitbison decides first, deterministically.** Your organization's rules
  grant what they grant, the registry's analysis says what a build did, and
  your policies accept what you have accepted. None of that needs a model,
  so a pull request whose changes the rules settle, or a build that is
  clean, never reaches Claude.
- **Claude takes only what is left**, within limits you set: it reviews an
  uncovered dependency, audits a release the registry has not reviewed
  (approved at once below your risk floor), judges whether an advisory
  reaches your code, and can decide whether a flagged build is a real risk.
- **A person is asked only for what is theirs**: an audit at or above the
  risk floor, overriding a registry review, another repository's code, or a
  decision you chose to keep for people. The summary links each one to its
  page in the Bitbison console.
- **Nothing blocks by default.** Every gate starts in report mode; you
  choose, per workflow, whether anything may stop a merge or a publish.

### How a pull request is reviewed

Bitbison reads every lockfile in the tree, whatever the ecosystem: cargo,
npm, yarn and pnpm, PyPI (uv, Poetry, requirements files), Go, RubyGems,
NuGet, Maven and Gradle, R and swamp. It compares the pull request's
lockfiles with the base branch's and takes the **direct** dependencies the
change adds or upgrades. A transitive release is granted under the direct
one that brings it in.

Each goes through your grant workflow: the check grants what your rules
grant; a request is made for each release still uncovered, with the pull
request as its reason; the check runs again. Claude starts only when
something is still uncovered after that. It reads the registry's analysis
of the release, comments inline on the manifest line, suggests a fixed
version or an alternative your organization already uses, and may submit an
audit, but never for a release the registry has already reviewed: that
review stands until a person overrides it.

Submodules are read from the two trees' gitlinks and `.gitmodules`, without
checking one out. Lockfiles inside a submodule are the submodule's, and are
read on neither side.

A developer whose coding agent has the Bitbison plugin asks for the grant
when the dependency is added (`bitbison run -- cargo add anyhow`), so the
pull request says **Already granted**. Without it, the review's request in
CI is the first, and the pull request says **Granted**. Both end the same
way.

Pull requests from forks get no secrets and are not reviewed.

### How a push is reviewed

A push is compared with the state Bitbison last recorded for the branch,
not with a commit, so nothing is fetched and a failed or missed run is made
good by the next. What the branch already pinned when it was first recorded
is its baseline, left to a person's `bitbison grant baseline`; the summary
says how many releases that is.

### What each branch and tag pins

Every push, tag, deletion and run asked for by hand records the state of
the ref checked out: everything its lockfiles pin, its submodules, and
every branch and tag the repository has, so a branch or tag that is gone
ends even if its `delete` event was missed. Leave `push` unfiltered: a
paths filter makes a branch look last seen when its lockfiles last changed.

### Vulnerabilities

The advisory table needs no agent and is always there. `vulnerabilities`
decides only when Claude is also asked whether an advisory is relevant to
your code. It searches before it reads, reads only around what it finds,
and for an affected or unsure advisory names the safest way out:

- **a compatible upgrade**: a fixed version within your version
  requirement, so only the lockfile changes;
- **a breaking upgrade**: a major bump, or a minor bump of a `0.x` version.
  Claude says the code may need changes and lists your uses of the package;
- **an alternative** your organization already uses;
- **a mitigation** in your code: avoid the API, turn off the feature,
  validate the input.

Raise the threshold to spend less: `critical` asks only about the worst.
This repository runs with `low`, so its demo's low advisory is judged too;
`high` is the usual choice.

### The build report and the gate

The gate waits for the analysis of every job of the run's build, posts the
report, and its job's outcome is its decision, so a job that `needs` it
runs only when it passes.

The report compares each job with the default branch's last build: a
finding or a host that is new stands out from what the build always does.
The runner's own connections, the checkout's and Bitbison's are folded
away. A package the build resolved that carries an advisory is listed under
**Vulnerabilities**, worst first, direct or not, and is a finding too, with
whether it affects the build or what was shipped: a build with one is not
clean until the package is upgraded or the finding accepted.

In this demo the gate runs on `master`, on dispatched runs and on pull
requests from branches named `build/...`, so every other pull request's
conversation is its dependency review alone. In your repository, drop that
condition to gate every build.

### The reviewer's token

A CI token with `mcp/ci/runtime` reads the registry and each run's analysis
report, requests grants and submits audits as an agent, and lets the build
analysis follow a finding into the build's capture, through views that mask
command lines, paths and other free text. It reads nothing of your hosts. A
token with `mcp/ci` alone does both reviews from the analysis report.

The reviews run in jobs of their own, never on your build's runner, and
only they hold the Claude credential and the CI token. They comment as
`github-actions[bot]`. The review's instructions come from your Bitbison
instance, which keeps them current.

### Without the build integration

The dependency review works on its own, with the workflow of step 2 and the
secrets of step 1. The gate needs a Bitbison build to read, so without the
integration it has nothing to gate.

### More

Every input of the two workflows, the runner, the model and workload
identity federation are in
[bitbison-labs/github](https://github.com/bitbison-labs/github)'s README.
