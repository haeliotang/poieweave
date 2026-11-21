# Resource Lifetime

Resources are held until group commit. Serialized same-robot tasks are scheduled
without overlap (T05/R01 makespan 18000). Parallel distinct-robot tasks share
group start and commit at group makespan (R00 10800). Step-level release remains
`NOT ESTABLISHED`.
