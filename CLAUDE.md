# FleetRanger

Self-hosted flight log analysis for DJI Enterprise fleets. Read `docs/architecture.md` for the decisions made so far, and `docs/roadmap.md` for status and open questions.

## Working agreements

- Ask before making architecture decisions. Don't assume.
- Keep the roadmap current: update the status and "Now" when work starts or finishes, and record new decisions in `docs/architecture.md`.
- Stay generic. Don't build customer-specific features ahead of need, and don't over-engineer.
- No customer data in this repository: no real logs, locations, names or serial numbers. The repo will be public.
- Raw logs and full telemetry are never exposed to users.
- When the DJI log parser shows bugs, ask the user; they have fixes from earlier work.

## Environments

- The dev VM is arm64 and disposable; sudo is fine there.
- Servers are x86_64 and managed through a separate Ansible repo. Change them through Ansible, not by hand, and only run read-only commands over SSH unless agreed otherwise.
