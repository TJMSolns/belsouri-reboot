# ADR-003: License Server Hosting

**Status**: Proposed
**Date**: 2026-03-03
**Deciders**: Tony (Product Owner), Claude (Developer)
**Category**: Architecture / Operations

---

## Context

The License Server is an external system operated by Tony that:

- Signs Ed25519 license keys for paying customers
- Is called manually by Tony to issue keys (not invoked by the Belsouri app at runtime)
- Must remain available for Tony to issue and renew licenses, but is not a critical path for the app itself
- Works fully offline after a license is issued — the Belsouri app validates licenses locally and does not depend on server uptime
- Expected volume: tens of practices, a handful of key issuances per month
- Tony is a developer but prefers minimal operational overhead
- Must be reachable from wherever Tony is working (potentially multiple locations)

The technical cryptographic requirements (Ed25519 signing, key format, payload schema) are documented in ADR-002. This ADR covers only the hosting platform choice.

---

## Decision: Three Options Evaluated

### Option 1: Fly.io (Containerized Service, Managed Platform)

**Overview**: Deploy a containerized License Server on Fly.io, a modern platform-as-a-service optimized for small teams.

**Setup**:
- Docker image containing the License Server (Rust/Python/Node.js)
- `fly.toml` configuration (one-time setup)
- Deploy via `flyctl deploy` or git push
- Fly.io handles routing, SSL/TLS, auto-scaling, backups

**Cost Estimate**:
- Compute: ~$5-15/month (3x shared-cpu-1x 256MB RAM instances in US + one backup region, or single instance ~$5)
- Database (if using Fly Postgres): ~$20/month (optional; not needed for this use case)
- Bandwidth: negligible (manual issuances, not high-volume)
- **Total: ~$5-15/month** (lighter than AWS Lambda if usage stays low)

**Ops Burden**:
- **Deployment**: `git push` to Fly or `flyctl deploy` from CLI (30 seconds)
- **Monitoring**: Fly.io dashboard shows recent deploys, error logs, CPU/memory
- **Secrets management**: Environment variables via `flyctl secrets set`
- **Scaling**: Automatic for low-traffic; manual region management if needed
- **Backups**: Configure via Fly volume or external backup (manual responsibility)

**Deployment Complexity**:
- Write `Dockerfile` (simple Rust web service)
- Create `fly.toml` (boilerplate)
- One-time `flyctl auth` and `flyctl launch` setup
- Straightforward rollback (revert to previous release via CLI or UI)

**Fit for Low-Volume Use Case**:
- **Excellent fit**. Fly.io excels at hobby/small-team projects with bursty traffic
- No need to scale — set and forget
- Global edge in ~20 regions; Tony can work from anywhere
- Very reliable SLA for a small operation (99.9%)

**Advantages**:
- Minimal ops overhead — modern PaaS designed for developers
- Transparent billing; easy to predict costs
- Fast deployments (30s)
- Good observability (logs, metrics in dashboard)
- Integrates well with git-based workflows

**Disadvantages**:
- Vendor lock-in (Fly.io-specific tooling)
- Less control over infrastructure than VPS
- Egress data costs if future integrations require large downloads

---

### Option 2: AWS Lambda (Serverless, Pay-Per-Use)

**Overview**: Serverless function that runs only when invoked, scaling to zero when idle.

**Setup**:
- License Server code as a Lambda function (Node.js/Python/Rust via custom runtime)
- API Gateway to expose HTTP endpoint
- CloudWatch for logs
- Deploy via SAM CLI or Terraform

**Cost Estimate**:
- Lambda: ~$0.20/month (40 invocations/month × 10s execution × $0.0000166667/GB-sec, 256MB RAM)
- API Gateway: ~$0.50/month (40 invocations × $0.0000035/request)
- CloudWatch logs: ~$0.50/month
- **Total: <$1/month** (incredibly cheap for low-volume)

**Ops Burden**:
- **Deployment**: SAM CLI or manual zip + AWS console (2-3 minutes, less trivial than Fly)
- **Monitoring**: CloudWatch Logs (requires AWS CLI or console login)
- **Secrets management**: Lambda environment variables or AWS Secrets Manager
- **Cold starts**: 1-3 second latency on first invocation after idle period (acceptable for manual key issuance)
- **Backups**: Code is versioned in git; AWS keeps immutable function versions

**Deployment Complexity**:
- Write Lambda handler wrapper (small boilerplate)
- Use SAM template (YAML configuration)
- Deploy via `sam build && sam deploy` or AWS CLI
- Rollback requires redeploying a previous version (manual process)

**Fit for Low-Volume Use Case**:
- **Good fit, but not ideal**. Lambda excels at event-driven workloads and truly sporadic usage
- No persistent state needed (License Server is stateless)
- Latency acceptable for manual issuances (cold starts not a problem)
- Scaling is automatic and transparent

**Advantages**:
- Cheapest option ($1-2/month vs $5-15/month)
- Scales to zero — pay only for what you use
- No server to manage; AWS patches and maintains
- Stateless design aligns with License Server architecture
- Good for occasional manual invocations

**Disadvantages**:
- Higher complexity to understand AWS infrastructure (Lambda, API Gateway, IAM roles)
- Cold start latency (1-3s) for first invocation after idle
- Vendor lock-in to AWS
- Harder to test locally without AWS Lambda emulation tools (SAM local)
- Requires AWS account and credentials management

---

### Option 3: Simple VPS (Hetzner or DigitalOcean)

**Overview**: Rent a basic virtual private server and run the License Server directly.

**Setup** (DigitalOcean example):
- Create a $5/month droplet (1GB RAM, 1 vCPU, 25GB SSD)
- SSH in, install Node.js/Rust toolchain
- Clone License Server git repo
- Run a process manager (`systemd`, `supervisor`, or PM2)
- Configure firewall (SSH port, HTTP/HTTPS)
- Point DNS to the droplet IP

**Cost Estimate**:
- Droplet: $5-6/month (smallest viable: 1GB RAM, 1 vCPU)
- Backups (optional): +$1/month
- Bandwidth: ~$0 (well under free egress)
- Domain/DNS: $0-12/year (optional; can use IP directly)
- **Total: ~$5-7/month**

**Ops Burden**:
- **Deployment**: `git pull && cargo build --release` or `npm start` on the droplet (2-5 minutes)
- **Monitoring**: SSH in to check logs (`journalctl`, application logs)
- **Secrets management**: Store Ed25519 private key in `/opt/license-server/.env` or systemd environment
- **Persistence**: Process crashes require manual restart unless set up with systemd or PM2
- **Backups**: Manual (copy droplet snapshot, git repo, .env file)
- **Security updates**: Manual (`apt update && apt upgrade`)
- **SSL/TLS**: Either use Let's Encrypt (free, requires renewal script) or self-signed

**Deployment Complexity**:
- Full control; can install any toolchain
- Write a systemd unit file (boilerplate; ~20 lines)
- Use git hooks or a deploy script for automated pushes
- Rollback: `git revert && systemctl restart license-server`

**Fit for Low-Volume Use Case**:
- **Acceptable, but overkill**. VPS is designed for sustained workloads, not bursty manual issuances
- Much more control than PaaS, but at the cost of management overhead
- Ideal if you ever need to run other services on the same machine
- Suitable for a developer who is comfortable with Linux and wants full autonomy

**Advantages**:
- Full control over the machine and network
- Cheapest if bundled with other services
- No vendor lock-in; can migrate to another provider easily
- Most familiar to developers with Linux/DevOps background
- Can run multiple services on one droplet

**Disadvantages**:
- Manual security patching and dependency management
- More operational burden (uptime monitoring, restart scripts, backups)
- Requires Linux knowledge and troubleshooting skills
- Single-point-of-failure (unless you add redundancy)
- Cold start (process restart): 5-10 seconds if it crashes

---

## Recommendation

**For Tony's use case (occasional manual key issuance, minimal ops burden, familiar developer), Fly.io is the strongest choice.**

**Rationale**:
1. **Low operational overhead**: `git push` deploys, dashboard shows health. No SSH required.
2. **Moderate cost** ($5-15/month): Predictable, transparent billing. Cheapest option (Lambda) requires more AWS expertise.
3. **Fast iteration**: Deploy in 30 seconds. Rollback is one click in the UI.
4. **Good for low-volume**: Fly.io's sweet spot is exactly this — small apps with irregular traffic.
5. **Observability**: Logs and metrics are visible without terminal work.

**Second choice**: AWS Lambda if Tony is already comfortable with AWS and wants the absolute cheapest tier. Trade-off: higher initial complexity, cold starts, less intuitive rollback.

**Third choice**: VPS (Hetzner or DigitalOcean) if Fly.io is ever unavailable or if the License Server will eventually be bundled with other services.

---

## Implementation Notes (Post-Decision)

Whichever platform Tony chooses, the License Server code should:

1. Be a simple HTTP service (e.g., Rocket/Axum in Rust, or Node.js Express)
2. Expose a single endpoint: `POST /sign-license` (body: `LicensePayload`, response: base64url-encoded license key)
3. Require authentication (API key in request header) to prevent abuse
4. Log all signature operations (who issued what, when) for audit
5. Store the Ed25519 private key securely (environment variable or secrets manager, never in code)
6. Be fully stateless (no database or persistent state needed)

Post-MVP: Consider adding a web UI where Tony can paste a `LicensePayload` JSON and copy the signed key (saves manual CLI use).

---

**Related**:
- ADR-002: Licensing Cryptography (Ed25519 signing, key format)
- PDR-001: Licensing Model (product decisions)
- `doc/domain/aggregates/license-aggregate.md`

**Reviewed By**: (Pending Tony's decision)
