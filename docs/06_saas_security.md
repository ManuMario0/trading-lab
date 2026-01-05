# SaaS Security Architecture & IP Protection

**Status:** Draft / Conceptual
**Objective:** Architecture for securing User Intellectual Property (Strategies) in the SaaS platform.

---

## 1. The Core Problem: The "Trust Gap"
Quants are extremely protective of their source code ("The Recipe"). They want the infrastructure advantages of the platform but fear IP theft.
**Goal:** Minimize the "Surface Area of Trust" to the absolute minimum required for execution.

---

## 2. The "Zero-Retention" Build Pipeline
The industry standard for secure CI/CD is the **"Build & Burn"** model. We promise (and architecturally enforce) that source code never touches a persistent disk.

### Workflow:
1.  **Ingestion (Volatile):** 
    - User source is pulled via one-time token into an **Ephemeral MicroVM** (e.g., Firecracker, AWS Lambda).
    - The file system is mounted as `tmpfs` (RAM Disk).
2.  **Compilation (Isolated):**
    - The Rust compiler runs inside this isolated sandbox.
    - **Optimization:** We cannot use persistent build caches (risk of leakage). We accept slower builds (~2-4 mins) as the "Security Tax".
3.  **Artifact Extraction:**
    - Only the final compiled binary (or `.so`) is extracted.
    - It is stored in an encrypted Artifact Registry (S3 + SSE-KMS).
4.  **Destruction:**
    - The MicroVM is terminated immediately.
    - Memory is wiped.
    - **Guarantee:** No trace of source code remains on our servers.

---

## 3. Runtime Security
Even without source code, "Execution Patterns" can theoretically reveal strategy logic.

### Measures:
1.  **Process Isolation:** Each strategy runs in its own container/namespace.
2.  **Network Air-Gap:** Strategy containers have NO internet access. They can only speak ZMQ to the `System Orchestrator`. They cannot exfiltrate data.
3.  **Memory Encryption (Future Roadmap):** 
    - **AWS Nitro Enclaves / Intel SGX:** Hardware-level isolation where even the root user of the host machine cannot inspect the memory of the guest enclave.

---

## 4. Encryption Strategy (At Rest)
If we *must* store source code (e.g. for user convenience/IDE features), it must be:
- **Client-Side Encrypted:** The user holds the key. We store blobs we cannot read.
- **Just-in-Time Decryption:** Decrypted only in the Ephemeral Build VM (which has the user's key injected for that session only), then wiped.

---

## 5. Trust but Verify
Tech alone isn't enough for Hedge Funds.
- **Legal:** Strong IP Non-Disclosure guarantees.
- **Audit:** SOC2 Type II compliance demonstrating that our admins cannot access build artifacts.
