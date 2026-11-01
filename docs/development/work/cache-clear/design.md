# Selective cache clearing

The owner requested a cache-management command, limited to clear, and confirmed
that only taloon-related cache may be removed. The presented command proposal
uses the existing configured/default cache location, works outside projects,
and succeeds when no cache is present. Implement inline in this session and
create a PR, following the existing execution preference.

## Approaches and decision

Deleting the configured directory recursively is rejected: TALOON_CACHE_DIR
may point at a directory also containing unrelated data. Selective deletion of
recognized completed cache records is chosen. A new dedicated storage namespace
would require migration and does not help clear current records.

## Behavior and ownership

`taloon cache clear` resolves the same cache directory as check, without lock,
policy, credentials or network access. Delete only immediate regular files
whose complete canonical filename matches the current v1 archive/index,
wheel/sdist, SHA-256 cache-key grammar. Content need not parse: damaged cache
records must also be clearable. Keep the root directory, unrelated/unknown
filenames, temporary files, directories and symlinks. Reject a root that is a
file or symlink; missing root and repeated clear succeed. Parent-directory
symlinks retain normal OS path resolution, as for check. This is a trusted local
cache operation, not an atomic defense against hostile concurrent path changes.

Print the removed-entry count on stdout; success exits 0, failure exits 2 with
a safe stderr message. No JSON mode, prompt, force option or per-project filter
is added. Cache scope is shared across projects using that location. Stop
concurrent checks before clearing: active writers can repopulate records and
there is no transaction or cross-process lock. A vanished entry during deletion
is harmless; other I/O failures fail the command and may leave partial progress.
Next online check reacquires evidence; offline checks can become incomplete.

Keep cache naming/deletion inside the binary-owned cache module, CLI wiring
and location resolution in CLI, with no I/O in the library and no new dependency.
The small filesystem operation needs actual process/symlink/preservation tests
and bounded filename properties, rather than an unrelated formal model.
