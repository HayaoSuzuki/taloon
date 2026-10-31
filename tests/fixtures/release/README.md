# Signature verification fixture

`reference-build.sigstore.json` is the original single-wheel public GitHub build
signature extracted without modification from hoimin's
`tests/fixtures/pypi-provenance-download.jsonl` at
`39d35f3980c428ab15258b519a233b86f1551ffa` (Elastic-2.0).
It is a cryptographic test witness, not a taloon release or taloon publisher.
Tests verify its authentic reference identity and reject taloon/wrong-source
identities, modified digests and signatures. No executable bytes are included.
