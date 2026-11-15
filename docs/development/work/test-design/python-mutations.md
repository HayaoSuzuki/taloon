# Python mutation verification after the test-design audit

Follow-up: [investigation and same-ID remediation of the 23 open candidates](python-mutation-remediation.md). The results below retain the initial-run snapshot.

Run on macOS arm64 against committed revision `73bfae0`, using
`uv tool run --python 3.14 --from <official-release-wheel> hoimin`.
The wheel was downloaded from the authenticated `tokyogas-tech/hoimin`
GitHub Release **v0.14.63** and kept fixed throughout plan and verify.
Wheel SHA-256: `3749dc199a2e775ac963f63aa3a4ae6e794dafba2692e09642507df56814e6d5`.

## Scope and results

Only production Python `tools/*.py` were mutation targets. Rust mutations and
Rust builds were not executed. The existing `target/debug/taloon` binary was
included solely for the Python installed-binary smoke tests.

The complete focused plan contained **1,689 candidates across ten modules**.
Selected **100**, ten per module: seeded shuffle (`random.Random(20261114)`),
then round-robin file selection. The exact executed IDs are listed below.
The remaining **1,589 were not verified**. This is a selected-run score, not a
whole-project or exhaustive mutation score.

- Normal baseline: **269 passed**, fixed pytest seed `20261114`.
- Completed verification: **70 killed, 30 survived**; raw selected-run score **70%**.
- **0 timeouts, errors, inconclusive, not-run, out-of-memory or process-limit outcomes**.
- Run ID: `71bfc669-1404-4cf8-82e4-7fcd1c242473`; terminal `complete=true`, exit code **0**.
  This tool's terminal result, rather than older skill prose about exit codes,
  establishes completion even though mutants survived.

## Execution and disk bounds

One worker; 8 GiB owned-workspace ceiling; 10 GiB mandatory free-space reserve;
1 GiB copy bound; 120-second baseline timeout; 60-second per-mutant timeout;
3,600-second total budget. macOS memory enforcement was explicitly best effort.
Fingerprint inputs included `pyproject.toml` and `uv.lock`. Tests and sources
were frozen throughout verify.

The native test argv was the repository `.venv/bin/python -m pytest -q -x
--no-cov --randomly-seed=20261114`. An isolated runtime TMPDIR avoided interference
with other projects' mutation leases. The dedicated temporary directory held the
wheel, plan and raw event stream and was removed by an EXIT trap.

Hoimin recorded **75.27 MiB peak owned bytes** and
**122.36 GiB minimum free space**.
Its execution cleanup was clean, with no disk-limit stop. The surrounding
temporary directory was also removed.

The optional wrapper summary initially treated hoimin's NDJSON stream as one
JSON document and failed after verification. An independent event reader had
already retained all 100 outcomes and the terminal event; these are the evidence
for this record. No verify rerun or result rewriting was needed.

## Survivor triage

Seven survivors appear runtime-equivalent under current callers: two private
validation exception-class substitutions caught identically by the caller;
a leading-newline YAML slice variation; two erased `typing.cast` type strings;
redundant `str` around a regex capture; and None/truthiness around
`shutil.which`, which returns None or a nonempty executable path. These are
provisional source-based classifications, not additional mutant kills.

At this initial-run snapshot, the other **23 survivors remained open**. Concrete missing behavioral expectations
include footnote-definition prefixes and diagnostic separators; distribution
target/wheel-tag constants; correct release-version URLs; bounded retry attempt
counts and delays; git commit-existence preflight; exact GitHub JSON arguments;
case-insensitive missing-release handling; and the wheel smoke cache-hit
assertion itself. Other survivors change exception types, container shapes or
ordering and require contract review before calling them equivalent.

No production code or tests were modified in this run. No assertion-based
100% mutation claim is made. Remediation and same-ID reverification were outside this initial run; see the
follow-up linked above for the subsequent results.

## Executed ID ledger

| Candidate ID | Target | Operator | Outcome |
| --- | --- | --- | --- |
| `m1_a625ef65766277571cb8184bf263ed349efed0255e57f0334d1ea5b6c5097630` | `tools/check_docs.py:21` | `exception_type_pair` | survived; provisionally equivalent |
| `m1_1389ba709fea666315d99d6b4308f45bcc901e5d3ced971a5df98de337ba224f` | `tools/check_docs.py:29` | `exception_type_pair` | survived; provisionally equivalent |
| `m1_c0a269bbce4d623230a39b3b2956095a3a07fa1250accdc567f7889550406fe2` | `tools/check_docs.py:33` | `collection_set_add_discard` | killed |
| `m1_e21e9443ecb285ce28be61ccde09c1c7f3c297927ff2b926347dcefe457dcfd2` | `tools/check_docs.py:35` | `string_segment_empty` | survived; open |
| `m1_80412427d9d5e43de607eb0c11388192c4c2d55deda325c81276efc0214b0e94` | `tools/check_docs.py:51` | `structure_slice_neighbor` | survived; provisionally equivalent |
| `m1_c773ebe2753ba80e194f2d0ce0d457a352822e11a36fc4e3c49d65e144dfe191` | `tools/check_docs.py:83` | `collection_list_tuple` | survived; open |
| `m1_25d7f7fcb59479e0c8ca48dfcca4a15e9af5bdf0b49b0802dd3d46e306d14211` | `tools/check_docs.py:87` | `statement_delete` | killed |
| `m1_69835c01e2d6b781d8cf66c59c3c4212aa367d97b738043386053da126f2f495` | `tools/check_docs.py:89` | `string_segment_empty` | survived; open |
| `m1_915307c3a6bad42ef08979ab0f6e9d668651d4c7ab1e6480fabdd527a1a9eb10` | `tools/check_docs.py:110` | `binary_mul_div` | killed |
| `m1_2983650929fafec1d55d82ce6c63dbfd9e8bba4c29b243263abbf4a9d6a7d241` | `tools/check_docs.py:112` | `binary_mul_div` | killed |
| `m1_ce12ca7ffbe17011b48278a160affab17871df1d7ce711c19725e740dd43bdb9` | `tools/json_data.py:9` | `remove_not` | killed |
| `m1_b4dda207b25229937604cded84a3b2bc377d9f9b8d6bc9c946ffef4556d4a78f` | `tools/json_data.py:9` | `boolean_and_or` | killed |
| `m1_514b07453a10f6e357b6ae74bad5339e8f39f27285e5d96139ead86e60957567` | `tools/json_data.py:9` | `collection_any_all` | killed |
| `m1_47623add8c3a4d82dc4a7fb7680e48c18f6f6ca7e88bf026c77a81873b559048` | `tools/json_data.py:9` | `generator_skip_first` | killed |
| `m1_d85b3b3482f8029e1a96244113bc1c9fe32dd78bb97194bb99892bfba596df26` | `tools/json_data.py:10` | `string_literal_empty` | killed |
| `m1_df6035f8d83f4bb8d92b013b9bb8c0464529d8f4138238772a042c50d7ee4b47` | `tools/json_data.py:11` | `exception_type_pair` | killed |
| `m1_4eb00c1ff1428770fb449e1226d24fb395e9d4a26b0249b12074d36808912698` | `tools/json_data.py:12` | `string_literal_empty` | survived; provisionally equivalent |
| `m1_8e99fe309d3e14a130604644615dd0376c08a6a7007e5ee1e389f7442fd4c59f` | `tools/json_data.py:17` | `remove_not` | killed |
| `m1_297f1ccaf4d708007277fbc8f25c1a36f8bd459fc490a78b28d75ea870834a37` | `tools/json_data.py:19` | `exception_type_pair` | killed |
| `m1_e1514ceacdec1a95e302f2cd6313d064dd321fd487c35f939194e730b6251250` | `tools/json_data.py:20` | `string_literal_empty` | survived; provisionally equivalent |
| `m1_7ef7f3ce18569975ba3dc440f21ec188f1899e1dca9cb06f894eddaafb35e97f` | `tools/knowledge_references.py:45` | `remove_not` | killed |
| `m1_5b01300bf4b98c558c5cb83ff7863f252bf5ba31069584901b39530707dd2660` | `tools/knowledge_references.py:48` | `binary_div_floor` | killed |
| `m1_dc7972fbd8b5a45323d634bb303548b36dc3ade392d6cfbcefa47aef9a463828` | `tools/knowledge_references.py:58` | `conversion_call_remove` | survived; provisionally equivalent |
| `m1_8aa8779d61ff9d595fb1cbb40ef2751427a8924224834d156b3f0e117865f169` | `tools/knowledge_references.py:61` | `compare_eq_ne` | killed |
| `m1_dfd6c997b81ec7679475b7d64df609ea2c3627074a3fdc1c83d5f2558daa7eef` | `tools/knowledge_references.py:79` | `collection_list_tuple` | killed |
| `m1_786675c695389d88ddc6ceb229c389d0ba6848618163e4c97067accb3cd61382` | `tools/knowledge_references.py:95` | `structure_mapping_get_subscript` | killed |
| `m1_fa482b596c3d3240b03c7eead63e416e32994683ada3e7600e4a95ee051dcd6a` | `tools/knowledge_references.py:113` | `collection_append_insert` | survived; open |
| `m1_36a8707071f7855bedd02cf62e94a4c55d35322e1b4836403d9c0b8f44ace482` | `tools/knowledge_references.py:186` | `collection_set_add_discard` | killed |
| `m1_cbbc8a7d26742e30062f8804b07f37095f5438a9ce4fe4b5f632d3e236a7b6db` | `tools/knowledge_references.py:209` | `string_literal_empty` | killed |
| `m1_1f1790ab436b1cb70e98b4966f8f4be146cf647ba6cbe0d0af51c732531c2ba2` | `tools/knowledge_references.py:209` | `string_literal_empty` | killed |
| `m1_52295abebb46576d7358576ce5dee9f86888767b960c59df3abba1a005cb4659` | `tools/pypi_provenance.py:94` | `membership` | killed |
| `m1_547e56fd0add41822e29e189fcdb0972d9671f4586b1703990cfff559a2d8cd8` | `tools/pypi_provenance.py:99` | `structure_mapping_get_subscript` | survived; open |
| `m1_d8d80d718281cc7704124a131625736ffd569825f0a99d5287eaf3af8857ae98` | `tools/pypi_provenance.py:140` | `string_literal_empty` | killed |
| `m1_8513ed22b7d6f280f1fe0ed396bce02a80ab3512b47437b3026bb125868b971d` | `tools/pypi_provenance.py:198` | `string_literal_empty` | killed |
| `m1_e6b0bb4bc14281c065ee2896fd28fa4ab27817cecf5fb8720005aa815e15b776` | `tools/pypi_provenance.py:263` | `structure_slice_neighbor` | survived; open |
| `m1_c87bc7af0287ed1f51605cdaf0bd9561a6140d2acb419cb76cd8c218a6a80ae5` | `tools/pypi_provenance.py:309` | `structure_mapping_get_subscript` | survived; open |
| `m1_16489dd76bca9d99515b79de60c8cca3dee59a582f7e2b2eb0f869488aeaa088` | `tools/pypi_provenance.py:323` | `string_literal_empty` | killed |
| `m1_30d8d06dcbb6e4269dcc28bc55d196ddd09281fd6f7c080a7b89b66af7cff028` | `tools/pypi_provenance.py:349` | `augmented_to_assignment` | survived; open |
| `m1_8ca5764f2a1a0fd2dd02698b8bdc3f5cdd45c1da3edbc70ff49f814cc8958c24` | `tools/pypi_provenance.py:364` | `integer_literal_neighbor` | survived; open |
| `m1_36234f86399da7db24e94b94eb8e3077d88043c458f677a7430fd15799300980` | `tools/pypi_provenance.py:389` | `string_literal_empty` | killed |
| `m1_9e30c07417c00f7554780af9f148c9dfde57164078a24c907ab05434b30b78c1` | `tools/pypi_release.py:45` | `structure_index_neighbor` | killed |
| `m1_cb3ce55b5df154b47aa59fbc0eb7b432277afe43a78b7d79d3b526d5b4e4109a` | `tools/pypi_release.py:47` | `string_literal_empty` | killed |
| `m1_11a773cb4f467e7f250562fa76088c6986a50dc5f7c078bab478a40fcef134ed` | `tools/pypi_release.py:54` | `collection_list_tuple` | killed |
| `m1_6363bf024fb2b6f8453dbc0d93db6dd1ff4dc3b0db00879ba29f5897a9b82244` | `tools/pypi_release.py:73` | `list_comprehension_to_generator` | killed |
| `m1_169720ec251f5ee3090990b3bbbf21dcf03b8c63b71b33f37608c162f1715fc2` | `tools/pypi_release.py:94` | `method_call_remove` | killed |
| `m1_d24e3d4d5b9e7a6d0872ae018d163c00a229776ba5efc45b0d5b63cfdd2781d5` | `tools/pypi_release.py:94` | `binary_div_floor` | killed |
| `m1_92db9ea2326e18bea75afe6f71b4cf30cc7137121c611e99f56c9504baa676e9` | `tools/pypi_release.py:96` | `string_segment_empty` | killed |
| `m1_99d4a68da584d25364878da43e01e97938d19fdb31bb69b0f93aa633053ce5c4` | `tools/pypi_release.py:120` | `identity` | killed |
| `m1_94ee4f4691055575188ce699466f4a2964bbe0ad31824e38b26be39af0742a28` | `tools/pypi_release.py:135` | `string_literal_empty` | survived; open |
| `m1_5618984c091af230bb6cfa991295bfe5b3bb30926f27b5479d23399b6a016207` | `tools/pypi_release.py:148` | `statement_delete` | killed |
| `m1_ab21ce9f860351f76153c5b5a77d9e520ec6e5b7ce555409e52283ee924f630b` | `tools/release.py:61` | `method_call_remove` | killed |
| `m1_e44dbcdea86af17053ca2eaccdeed2974de632d80086d81c5537b0552f4baa2c` | `tools/release.py:66` | `binary_add_sub` | killed |
| `m1_1476da7ab60e447224ba212e6512dde4f833fa0da99d02d58909c3d8e77f8048` | `tools/release.py:73` | `compare_eq_ne` | killed |
| `m1_44f89f994ead3595a2fdd32c971bfff68edd12aa23c2acc8a761ebdaa489c2d5` | `tools/release.py:88` | `statement_delete` | survived; open |
| `m1_b2f1b66fa541685453e27110e364d57f254efe8fa359a57b151b4f89811ca0e0` | `tools/release.py:100` | `structure_slice_bound_move` | killed |
| `m1_9b3dfd89d48402c0fda9ddc40980bf695c60898a4162adbb24aeae796a9138d9` | `tools/release.py:101` | `collection_any_all` | killed |
| `m1_53578aeaa01f28f9dd7c7fc1b7da32e902a91a0485fc6cb1ca45359b837ced99` | `tools/release.py:101` | `remove_not` | killed |
| `m1_c97cb85462123fef889d9a0835741f0d3a2d4f661a702c12b0b98d844e87d49c` | `tools/release.py:153` | `string_segment_empty` | killed |
| `m1_ed9736c9dc5346b629b70e41f2feed101d5416c7a158d7354c00e4f3b5143dd9` | `tools/release.py:189` | `compare_eq_ne` | killed |
| `m1_b7e3212a8591a13a7dca4161719acce6346b2986051f7e8b90ba691b35bb347c` | `tools/release.py:189` | `string_literal_empty` | killed |
| `m1_968c5ed79ed9a60a56114e7009aec0e2f3dce8f35d33fb6fc8d4a9265cc7aeea` | `tools/release_checks.py:18` | `boolean_and_or` | killed |
| `m1_9ae1fb46b362be9a4541604afb3ff7d32ff2924129fad49a81d1ba7377f09b08` | `tools/release_checks.py:25` | `string_literal_empty` | killed |
| `m1_e6c8591f483fe79f194826a11d78520607c9150ab666f6a30aa3bf099743b3da` | `tools/release_checks.py:38` | `structure_mapping_get_subscript` | survived; open |
| `m1_865c0cd824df15ab1e9b9900a0cba76322655b753c7d1de40888ca966babe910` | `tools/release_checks.py:52` | `boolean_and_or` | killed |
| `m1_c765ac225f89ba2073ebfab100027336acd1298970e3b710b300dcecbc7bc63b` | `tools/release_checks.py:59` | `remove_not` | killed |
| `m1_22032be73d58526a75f0b91f9fddec4d469eff42bc8f732044023cece3a0da95` | `tools/release_checks.py:63` | `none_truthiness` | survived; provisionally equivalent |
| `m1_61d290fd5359707104ac5017a1690b6475bed7949ca8a4d121f617ecbf3f8819` | `tools/release_checks.py:63` | `identity` | killed |
| `m1_ac2e9700dbaa067c23bb44a79ad4f3e26862acec9708a86d121d2a19f7754ade` | `tools/release_checks.py:83` | `generator_skip_first` | killed |
| `m1_cd398f0a15975da9e4eac59fda0434767ebbb8f7b484b0a3e8525bca771e6126` | `tools/release_checks.py:89` | `string_literal_empty` | killed |
| `m1_db7a6b3df45f5c3d7299be83102bf4d40ec0bc28734a252fa1f259e0acd9d1ad` | `tools/release_checks.py:98` | `statement_delete` | killed |
| `m1_99b991459dd43f23ed9ca181e6d9ce29cf8335fce6d6f7fe8c79100e02f806a1` | `tools/release_policy.py:24` | `string_literal_empty` | survived; open |
| `m1_724841045c971bf6aa21660bca93ed40fea180f9be2106087fbb8dafdd1500d9` | `tools/release_policy.py:24` | `string_literal_empty` | survived; open |
| `m1_c3d4373d9dca3c269044890a9736541d260e9092d50a47702c2d6181c6423234` | `tools/release_policy.py:24` | `string_literal_empty` | survived; open |
| `m1_b6e5069b2a24cdb39d9d349108ec88a05d832c3bc2a8097e2241ce911d3b54d0` | `tools/release_policy.py:29` | `string_literal_empty` | survived; open |
| `m1_86017abacea662706cc858adc3a21351f9a7c0ad043390258e5e5672afad10e0` | `tools/release_policy.py:30` | `string_literal_empty` | survived; open |
| `m1_204c82815f8f62d99c44289298d7f4d25c7bb45fc3cd6e0b7dd034280a8ea2ab` | `tools/release_policy.py:33` | `string_literal_empty` | killed |
| `m1_8ff2a4c8145e30b7075e5c1fa1c35bb2ecc3636b2af3bef265c636bba2086c2c` | `tools/release_policy.py:34` | `string_literal_empty` | survived; open |
| `m1_875167eb29eb4435c2d74ffce58ca5e0be265943fd919a9d5a6a5c2ac736c368` | `tools/release_policy.py:35` | `string_literal_empty` | survived; open |
| `m1_7f2f77287e3e9c4d6d7d6f4afe4bfda5781f5556543e70411f5f3ad7089d87ee` | `tools/release_policy.py:36` | `string_literal_empty` | survived; open |
| `m1_36dd97aa91198f5ad3d0c56ac784f725299182eb372c9ee4388f0dbf3a6364ec` | `tools/release_policy.py:39` | `collection_list_tuple` | killed |
| `m1_f7d30ada2da2c80c1c96a10ea686feedd8a1282ff5f5b2f0d7e6be68e5ffded9` | `tools/release_prepare.py:24` | `exception_type_pair` | killed |
| `m1_b99e411602e190b4340ac2ec2c789e15cfb0690feaeef6772e44ed096c23c610` | `tools/release_prepare.py:34` | `string_literal_empty` | survived; open |
| `m1_2d1a9645855969257042e2ff31405dcc15e5cc5ec2d6cc983a8635eaf7c8c5cf` | `tools/release_prepare.py:44` | `string_literal_empty` | killed |
| `m1_9401c390b38c8f6d5141662ce1bef65df590b6453e33ead315ea10d19cfe9c79` | `tools/release_prepare.py:50` | `method_call_remove` | survived; open |
| `m1_43beb490562a0d73acfcfbb84fe5964fadb7e174c435a8403902f053e9f29a8c` | `tools/release_prepare.py:62` | `method_call_remove` | killed |
| `m1_3c570ad13818db7ae824856e63a1eeb875971cb9ac6cb114b9e4173917d7f98b` | `tools/release_prepare.py:75` | `remove_not` | killed |
| `m1_ca9429bf519dd84fe03bd0195653f17e8ba65bdcdc1902f9aa8e1cf0cb0e94d8` | `tools/release_prepare.py:89` | `structure_slice_bound_move` | killed |
| `m1_4502b51544d039a858e34886fd5a35cfd93f7b93692dcb1c148224560e080bd0` | `tools/release_prepare.py:93` | `string_literal_empty` | killed |
| `m1_f4209930decccf187df644280db86ed0ebeb3acf837eccc4118c54f73fd19bf1` | `tools/release_prepare.py:95` | `method_call_remove` | killed |
| `m1_8c828355b11e47846f89961ff632bf418cf84a43d3aac5b0aec0edda06c2479e` | `tools/release_prepare.py:98` | `generator_skip_first` | killed |
| `m1_ae55f6ee966d1ae9d7c97aca850ebe003f2a9e04b1976878f56eb5d274c5f678` | `tools/wheel_smoke.py:31` | `binary_div_floor` | killed |
| `m1_2afcefbb7ae29711701958b51ade9b0c3e49f4baaaabaf992563a5aced15cb70` | `tools/wheel_smoke.py:58` | `boolean_literal` | killed |
| `m1_c8e3bf829786f9d98614e7fc862f5b88e76f85782a6fd4bc4238de06714dd69f` | `tools/wheel_smoke.py:63` | `structure_index_neighbor` | killed |
| `m1_c8aa961d32ec5230365c9ee57544ba5e06a7aef46b7e3d1c21d95dd0400426db` | `tools/wheel_smoke.py:67` | `method_call_remove` | killed |
| `m1_8dcdcb2fa7d9c63a60e4909c8869548be1763d6046228bcd6a0368b748a34b2b` | `tools/wheel_smoke.py:67` | `compare_eq_ne` | killed |
| `m1_115aaab137680f05b16588dcb65d7b516487828fed10888994fc4eea59a3fb65` | `tools/wheel_smoke.py:73` | `string_literal_empty` | killed |
| `m1_efe43f0d15f12d60881cbf7c228be4a21452e9f1ee7ec5314d9d93fdd24f8bac` | `tools/wheel_smoke.py:86` | `compare_eq_ne` | killed |
| `m1_227740e360e35e1515428a28aed45216b316d42cbb70fcc7ed97c7657aa43789` | `tools/wheel_smoke.py:94` | `compare_eq_ne` | killed |
| `m1_6c5b05671ccce2a1e13ee744d4fad4974d8c2080abdf2ea9680104d7d50aab32` | `tools/wheel_smoke.py:98` | `statement_delete` | survived; open |
| `m1_e07abd6fa66c98ec1ba44e6544f2d3c86e9fa27e6ae37f7370ab7ee092247ea9` | `tools/wheel_smoke.py:113` | `integer_literal_neighbor` | killed |

## Surviving changes

| Target | Original | Replacement |
| --- | --- | --- |
| `tools/check_docs.py:21` | `TypeError` | `ValueError` |
| `tools/check_docs.py:29` | `ValueError` | `TypeError` |
| `tools/check_docs.py:35` | `^\[\^` | `` |
| `tools/check_docs.py:51` | `4` | `3` |
| `tools/check_docs.py:83` | `[f"{root / 'index.md'}: missing OKF entry point"]` | `(f"{root / 'index.md'}: missing OKF entry point",)` |
| `tools/check_docs.py:89` | `: ` | `` |
| `tools/json_data.py:12` | `"dict[str, object]"` | `""` |
| `tools/json_data.py:20` | `"list[object]"` | `""` |
| `tools/knowledge_references.py:58` | `str(marker[1])` | `(marker[1])` |
| `tools/knowledge_references.py:113` | `lines.append(line)` | `lines.insert(0, line)` |
| `tools/pypi_provenance.py:99` | `record["digests"]` | `record.get("digests")` |
| `tools/pypi_provenance.py:263` | `1` | `0` |
| `tools/pypi_provenance.py:309` | `provenance.get("attestation_bundles")` | `provenance["attestation_bundles"]` |
| `tools/pypi_provenance.py:349` | `+=` | `=` |
| `tools/pypi_provenance.py:364` | `1` | `(2)` |
| `tools/pypi_release.py:135` | `".whl"` | `""` |
| `tools/release.py:88` | `git(root, "cat-file", "-e", commit + "^{commit}")` | `pass` |
| `tools/release_checks.py:38` | `run.get("head_sha")` | `run["head_sha"]` |
| `tools/release_checks.py:63` | `gh is None` | `(not (gh  ))` |
| `tools/release_policy.py:24` | `"x86_64-pc-windows-msvc"` | `""` |
| `tools/release_policy.py:24` | `r"win_amd64"` | `""` |
| `tools/release_policy.py:24` | `"win_amd64"` | `""` |
| `tools/release_policy.py:29` | `r"manylinux[^/]*x86_64"` | `""` |
| `tools/release_policy.py:30` | `"manylinux_2_17_x86_64.manylinux2014_x86_64"` | `""` |
| `tools/release_policy.py:34` | `"aarch64-apple-darwin"` | `""` |
| `tools/release_policy.py:35` | `r"macosx_[^/]*arm64"` | `""` |
| `tools/release_policy.py:36` | `"macosx_11_0_arm64"` | `""` |
| `tools/release_prepare.py:34` | `"--json"` | `""` |
| `tools/release_prepare.py:50` | `result.stderr.lower()` | `(result.stderr)` |
| `tools/wheel_smoke.py:98` | `require(                 condition=(object_mapping(report["acquisition"])["cache_hits"] == hits),                 detail=report,             )` | `pass` |

