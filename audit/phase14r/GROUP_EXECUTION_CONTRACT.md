# Group Execution Contract

- One group has one restoration target and 1–2 explicit tasks.
- Each task maps to a real ontology process, duration and resource set.
- The canonical scheduler chooses resources and start/end times.
- A group commits only after every task completes.
- If any active execution dependency fails before group commit, the group
  interrupts and no restoration is committed.
- Group-level resource hold is preserved; task A completing at 7200 does not
  release resources before group commit.
