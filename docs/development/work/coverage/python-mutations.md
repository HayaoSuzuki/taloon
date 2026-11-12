# Python mutation verification

Run on macOS arm64 in the working tree based on `b004dd3`, using authenticated
`tokyogas-tech/hoimin` GitHub Release wheel v0.14.63. Production targets were
`tools/*.py`; tests themselves were not mutation targets. Only tests changed.

## Selection and execution

The complete focused plan enumerated 1,689 candidates across ten Python files;
no discovery truncation occurred. Selected 116 distinct IDs, listed below.
The other 1,573 candidates were not verified: this is selected behavioral
verification, not an exhaustive mutation score or a claim that no bugs remain.

All plans used one worker, an 8 GiB workspace bound, a 10 GiB free-space reserve,
and macOS best-effort memory enforcement. Free space was checked before each
verify. Fingerprint input was `pyproject.toml`. `target/debug/taloon` was explicitly
included because the installed-wheel smoke tests require the real binary.
The normal command was the repository `.venv/bin/python -m pytest -q`.

Initial setup failed its worker baseline because the binary was omitted; the
plan was regenerated with that include. The first 100-candidate run finished
individual cases, but correctly rejected whole-run completion after tests were
edited during execution (`workspace.original.changed`). It is investigation
only, and its score is not used. Tests were frozen during every subsequent run.

The next plan selected those 100 IDs plus 16 candidates covering previously
unselected modules, with max-mutants 121 and total timeout 3600 seconds. It
produced 96 killed, 18 survived and 2 timed-out results; the timeout results made
that run incomplete. Replanning for four actionable survivors and two timeouts
used an explicit 60-second mutant timeout and relevant test files:
`test_knowledge_references`, `test_release_checks`, `test_distribution`,
`test_release_prepare`, and `test_wheel_smoke` under `tests/`.
That run killed four, retained one survivor and timed out one. The final plan
selected the last survivor, the timeout, and the blank-footnote candidate;
its argv selected only `tests/test_knowledge_references.py` and
`tests/test_release_checks.py`. All three were killed, with successful baseline,
completed verification, and exit 0.

The table is the **latest per-ID result ledger** across those runs: 103 killed,
13 survived, and zero unresolved timeouts/errors. The selections, budgets and
test revisions differ; no `hoimin progress` comparison or combined mutation score
is claimed. Every plan/report lived in its own immutable `mktemp` directory with
an EXIT cleanup trap. Runtime TMPDIR was isolated there to avoid stale global
workspace leases; all those directories were removed. Raw manifests/reports
were not committed.

## Behavioral gaps addressed

- Report every missing reference, unreachable concept, and CLI diagnostic.
- Classify scheme-only and host-only URLs as external rather than local paths.
- Resume scanning after a matching code fence, preserving following references.
- Continue after inline links and traverse all cyclic/duplicate navigation paths.
  A generated graph test uses independent Floyd-Warshall transitive closure.
- Keep unused footnote continuations out of navigation even when their recognized
  label normalizes to empty text; do not silently reclassify them as body links.
- Reject invalid Base64 characters in signed attestation statements.
- Propagate actual GitHub CLI failure and inspect subsequent bundles: a later
  invalid same-subject signature is fatal, and every wheel must be staged.
- Keep release platform names consistent with the independent workflow matrix;
  recovery CLI arguments must select `HayaoSuzuki/taloon` explicitly.
- Reject a missing wheel before installer access and fail the inventory command
  when the only issue is unreachable concepts.
- Replace real waiting in the workflow service fixture with finite virtual time.
  Breaking a success predicate now fails promptly instead of sleeping up to
  30 minutes. Only the tested module's time object is replaced, not Python's
  global time module.

The 13 residual survivors are equivalent over their actual return domains:
`Path | None` (all Path objects are truthy), fixed nonempty fence strings or None,
nonempty `shutil.which` paths or None, and `re.Match | None`. They need no tests
that assert a particular spelling of otherwise identical Python expressions.
No production changes or new coverage exclusions were made for these mutants.

Final validation: 265 Python tests passed, with source and test line coverage
remaining 100%. Ruff formatting/checks, strict ty and repository documentation
validation passed. These are working-tree results; no hosted run is claimed.

## Target fingerprints

- `tools/check_docs.py`: `062aa81ba49bd1ede1b375affece5739db4ee7d9ed0b9c81cc630c9630f5ce0c`
- `tools/json_data.py`: `790e22fc52b7fbbe63f5bbf6d6cca0774e97ba72488d2786de5b554cde222eb5`
- `tools/knowledge_references.py`: `cd2840588a52c5d08eaaa2f39ee53ddca138f76d57afa9927b7fa21756c4bcda`
- `tools/pypi_provenance.py`: `946d411a072500c2a92e4141c0714e75d43f963d4849d785f3a809e68a42b9d5`
- `tools/pypi_release.py`: `a3af66a16dd474ef02218149b04c8fe373613fb0f1f3f1fbe2a00aae70cee1ca`
- `tools/release.py`: `18a25e4e15bed73867db26f2ccbe258e4faa9868489cd2097b50294155ba6790`
- `tools/release_checks.py`: `f1558a1f9e415e4856698e7b2d71d24465a1cb3539326e4d0d52eea01b68633e`
- `tools/release_policy.py`: `7ffab403427a70c2b11831451fb5932eb605cebae6be9a9a8bc9a8b1c7d0cb48`
- `tools/release_prepare.py`: `9a4b05e8b9afa56c46c1a80b1d196748781e91b47a0ee03fed79c603309f0154`
- `tools/wheel_smoke.py`: `50b47d4b46a566e259dcac2152029268ccfd089bd276c3f4af3ea1a81a641089`

## Selected candidate IDs and final status

| ID | Target:line | Operator | Latest status |
| --- | --- | --- | --- |
| `m1_427173d8d93cdedaadf23345c9d669cc8afda5dd5fa5306e735af3806d9fa216` | `tools/check_docs.py:19` | `remove_not` | killed |
| `m1_95217ddcb17e63b1073d41bb5b8e8059863470ae27a371248e769acde0dfca68` | `tools/check_docs.py:23` | `remove_not` | killed |
| `m1_c8a3a982b7f9de7ae3285e7c6becb8078ab36f651002713b05ed956731578e92` | `tools/check_docs.py:27` | `remove_not` | killed |
| `m1_b9697f24da0ba44ec8808cd402fb3897e187b237d1735e23e5cde3c952ebd7cb` | `tools/check_docs.py:30` | `membership` | killed |
| `m1_90a95deeaeaa518067c7ae8e97b138d0b781cd48839d3270b7856f6e0d468a2f` | `tools/check_docs.py:34` | `membership` | killed |
| `m1_17cffc261bfce2dcd81e8df6e4dab8023828172f31ba65e855eedf70167b7959` | `tools/check_docs.py:43` | `membership` | killed |
| `m1_07fdecdcf48cac0e68a9303b870b0a891590ddc8fc50916fdec728a5865cc024` | `tools/check_docs.py:48` | `remove_not` | killed |
| `m1_8b0d199f9be29f3896ab42932be9fd86c8bb0eb4a922e0e77a45cd22feb6da28` | `tools/check_docs.py:53` | `remove_not` | killed |
| `m1_ff17b9d6f0acfea2d83dc30e27f10f90a3b5d077b27b6679ef8b80789e0c1562` | `tools/check_docs.py:56` | `compare_eq_ne` | killed |
| `m1_aec4d0c203d82b54f7e9f1d73f23d1abfbce62c0945f0037490d0b5315885310` | `tools/check_docs.py:57` | `compare_eq_ne` | killed |
| `m1_9acb68ad9952a050f996d1d843f950a51e008a11fc79d7a7c8e1ea7a172ddcd8` | `tools/check_docs.py:62` | `remove_not` | killed |
| `m1_0c845a4f49b5578cd7bbe106dae6e3f53c173a7d29efe7f1af8723511d931996` | `tools/check_docs.py:66` | `remove_not` | killed |
| `m1_30eb3444ec0d27d3565bf3423bcf7058f6d7630660f724295a2e53728f8db543` | `tools/check_docs.py:71` | `generator_empty` | killed |
| `m1_d57fc4985907fd8af95a38946c66d7de17622fc62e3fa4dec1ee24b1e1721a18` | `tools/check_docs.py:73` | `remove_not` | killed |
| `m1_15526c2925b25562454e07abdcabd9de141860677a9d02c36de25aef0b2bd0ff` | `tools/check_docs.py:76` | `loop_break_insert` | killed |
| `m1_dde252cf9de3a6c2c41846e2320b0573aa5731dc749d1a7fbabe4cba9c88e145` | `tools/check_docs.py:82` | `remove_not` | killed |
| `m1_5bbd9e624ad603c30cecf27d91aa4e2c022cfb829f7b0575b5cafa9d16273a5f` | `tools/check_docs.py:89` | `loop_break_insert` | killed |
| `m1_64671f00a1c4988c8e028c2c7418b5972e684c7f228c5768d1695ad185bf0bef` | `tools/check_docs.py:104` | `remove_not` | killed |
| `m1_08206fa5661d0860c561fed76c1282496fc7836dcd036cb8f85ae4cba5df1106` | `tools/check_docs.py:110` | `loop_break_insert` | killed |
| `m1_1504e5b4f3501af9f576f232a73a9ae7c126ff59948c532bb80723f93973ca20` | `tools/check_docs.py:112` | `loop_break_insert` | killed |
| `m1_b45fd303b98263c9d1e6480233ea0ed0627bcd29c4b6f1e9fce501474c1a90d8` | `tools/check_docs.py:115` | `generator_empty` | killed |
| `m1_1d14306e7d941387d1132866533dcd48c13041e5ec07254ea8de4b4c4a0d2e2a` | `tools/check_docs.py:119` | `break_continue` | killed |
| `m1_7dfd55f32efae58f97389a09ab8fc4416742d1ba31035df14bcf96a4f5725858` | `tools/check_docs.py:123` | `none_truthiness` | survived (equivalent) |
| `m1_016b6d42d2b3034781dd374a5c4e5ef65ae4b6501cc73fdb1b9b13474a8cf5e5` | `tools/check_docs.py:124` | `loop_break_insert` | killed |
| `m1_c191dda8dd99cf7b36cc5bddcff186fdf5d4c7b6e29051ebe55cf6881502d5a4` | `tools/check_docs.py:132` | `loop_break_insert` | killed |
| `m1_ce12ca7ffbe17011b48278a160affab17871df1d7ce711c19725e740dd43bdb9` | `tools/json_data.py:9` | `remove_not` | killed |
| `m1_8e99fe309d3e14a130604644615dd0376c08a6a7007e5ee1e389f7442fd4c59f` | `tools/json_data.py:17` | `remove_not` | killed |
| `m1_e4c890364697011213409973278b4f2a58b0a5a69528876541f1607f18c72569` | `tools/knowledge_references.py:42` | `boolean_and_or` | killed |
| `m1_7ef7f3ce18569975ba3dc440f21ec188f1899e1dca9cb06f894eddaafb35e97f` | `tools/knowledge_references.py:45` | `remove_not` | killed |
| `m1_c140aef1b5f804bf0e02c1a8626de9172a91b31937653003fe1d4030aa625a40` | `tools/knowledge_references.py:54` | `generator_return_insert` | killed |
| `m1_3a8f55d7a5727e9d68009df8843109397eb7e830396acc4c635c1dae0a9bb287` | `tools/knowledge_references.py:59` | `none_truthiness` | survived (equivalent) |
| `m1_8aa8779d61ff9d595fb1cbb40ef2751427a8924224834d156b3f0e117865f169` | `tools/knowledge_references.py:61` | `compare_eq_ne` | killed |
| `m1_5c1e40c7a9af754c37832cccd6f6a0f4a42796fea9cf856c5fafa7d3ac9c7ae4` | `tools/knowledge_references.py:63` | `break_continue` | killed |
| `m1_35de22af100f0cd2ec0552f1f5e4c06fb15e758c8653feee567c9f26d6e68a37` | `tools/knowledge_references.py:64` | `none_truthiness` | survived (equivalent) |
| `m1_d3ff22857575edd48cfaccf0fc9f565947b14cdd66885ff06e51c1b0d9d0b243` | `tools/knowledge_references.py:65` | `loop_break_insert` | killed |
| `m1_b062fb73c1dd21fbc5d392fac78eea1a50fa870a0707e6a049069c2b0b83dce5` | `tools/knowledge_references.py:86` | `break_continue` | killed |
| `m1_15bd7c1416e29d388a4b62e0bb33c4e5ac5884612bcfc74e4ff0657bdb22b28b` | `tools/knowledge_references.py:87` | `none_truthiness` | killed |
| `m1_7f1b45543d18653704732327acfe86f198d8127ecffdabdcef8b0a6ca9c97da9` | `tools/knowledge_references.py:89` | `break_continue` | killed |
| `m1_c02e18d295761dab565f78bdbbf75b7ba863c918da09d829efde0a59ac177d37` | `tools/knowledge_references.py:96` | `boolean_and_or` | killed |
| `m1_e43de1234923f8994befbb18e8c9593821e1a114f71e191386e9ca6642c42830` | `tools/knowledge_references.py:98` | `loop_break_insert` | killed |
| `m1_6eea1daecc46312ad2dded0610801a43e778b75ddcc220624e73ca725d004a8d` | `tools/knowledge_references.py:111` | `while_condition_false` | killed |
| `m1_b0a01850b2c6e9929bfd5c4712e0a77e467be805c26ebba4d9863858c42671c0` | `tools/knowledge_references.py:116` | `membership` | killed |
| `m1_1690eff55fab9be9e5a7422a3c9d6e6fa92540482607681366660c3b183f239a` | `tools/knowledge_references.py:118` | `loop_break_insert` | killed |
| `m1_26f1fa41e7f932bb46720812450e34a8761a9ce2bd77bb448248ae80966e6c80` | `tools/knowledge_references.py:129` | `generator_return_insert` | killed |
| `m1_f580198509fd6b8964f3dadae80880d886985a89bf6ee171af4bdb1a5c6862d1` | `tools/knowledge_references.py:137` | `boolean_and_or` | killed |
| `m1_cd23635e33ac8e8d80b0a96e5e22a84f93e7813ceda13a1d5c6b77844c139eaa` | `tools/knowledge_references.py:139` | `remove_not` | killed |
| `m1_c53a3a6cdb5db9f862ac7deb3cc8d14a146d372a5d4ce3d8fb147b1d8060546b` | `tools/knowledge_references.py:140` | `yield_from_to_yield` | killed |
| `m1_6502dcb16572b4cebcee1b9207cf3fcb205d3894742bb826eff6f5e0b3351d2f` | `tools/knowledge_references.py:147` | `break_continue` | killed |
| `m1_72d571ad74de3879c5ab4dac22636a64d35b6e6e8a3eccd8974c221b45ca914b` | `tools/knowledge_references.py:148` | `boolean_and_or` | killed |
| `m1_33d6a80d8bb841d21a004a73ed41c1957bffa376a0b05df6de6b1e61e819d0fc` | `tools/knowledge_references.py:149` | `membership` | killed |
| `m1_92de151a98128d8aa591a9f04654c743ba89a989e84cf2a1c7197bd3ffbc07f6` | `tools/knowledge_references.py:151` | `loop_break_insert` | killed |
| `m1_06ffe04a8c2d611db8014dc3ed8d5e96651267fce58da43b73678eac94240088` | `tools/knowledge_references.py:157` | `remove_not` | killed |
| `m1_a8970e8f15b414d5f3a86a5b47e33d1ef2edf2ca8ec25bb5f8d923939a375508` | `tools/knowledge_references.py:170` | `boolean_and_or` | killed |
| `m1_342dce43216d6c240bbc96066517684ea031fea2bd1bef64c42a7eca0850cd3e` | `tools/knowledge_references.py:179` | `none_truthiness` | survived (equivalent) |
| `m1_9ce649f1704fcd9e3ad8484f4c6069a08e43a6cbec6efa9d6823559361b49c5a` | `tools/knowledge_references.py:180` | `none_truthiness` | survived (equivalent) |
| `m1_69637e108a99b659b62c6961bb62f0d2fd1b3db14d78147c1276d8a0d7bac413` | `tools/knowledge_references.py:182` | `loop_break_insert` | killed |
| `m1_eb46795297f4e8d9475305084bd26d406d82d8797be9a88b2d07f4df8b09d060` | `tools/knowledge_references.py:183` | `boolean_literal` | killed |
| `m1_2f4c2f6834e07f0b58f0efd7ec4589b144d8ddcec48a803ab5696e6f581f8846` | `tools/knowledge_references.py:185` | `none_truthiness` | survived (equivalent) |
| `m1_79213c176bc79384deddd33c1ece50a18df4a18e691e2883f32a500d5a4cc253` | `tools/knowledge_references.py:186` | `loop_break_insert` | killed |
| `m1_b5591b6024d5ca7f55d4ef85873dcfa4b3b87510113d904b569296f281847ad1` | `tools/knowledge_references.py:193` | `none_truthiness` | survived (equivalent) |
| `m1_41104af5a5a80592eba75577fd0ac54810f1c324681b11548e1966f9851e1772` | `tools/knowledge_references.py:194` | `none_truthiness` | survived (equivalent) |
| `m1_81e918286d4e2d74eaee6090913e926b11b078069e26bbd76addafd9a64a0cb8` | `tools/knowledge_references.py:196` | `loop_break_insert` | killed |
| `m1_1eaaaf6a55ce96309f30895d6a75274340e0657ef5300d7f284023493e931d70` | `tools/knowledge_references.py:199` | `while_condition_false` | killed |
| `m1_f9225c436e8813dcb24ddb1cc4f1db6b5905698e2b66a06be390502d563eaa35` | `tools/knowledge_references.py:201` | `membership` | killed |
| `m1_5e1277319a24f1b72f2668186e376c610bbfab140f0702679bf51bcb023e13f9` | `tools/knowledge_references.py:202` | `break_continue` | killed |
| `m1_e627fb1a305b2f443662daa1d8f5894b575d9ed4a944c91511eda12bc8dc2902` | `tools/knowledge_references.py:204` | `loop_break_insert` | killed |
| `m1_4624ab335563224f3fe4fa3493b9b06ab20ca034dcae5500e5da4555a6042093` | `tools/knowledge_references.py:209` | `identity` | killed |
| `m1_4274a916ba1c800d692a9f60e621ef3cfa5608bc4c875d98b195fc63efba734a` | `tools/knowledge_references.py:210` | `identity` | killed |
| `m1_d5adb1ddd7b60e2c95824074ecb5b3f46dd1d93569dbcfa9ad30c900ca78cbea` | `tools/knowledge_references.py:212` | `membership` | killed |
| `m1_414880ca7c350297d820ae5e381f35d6e49b3ebd76aa806a88152885a3d6ad35` | `tools/knowledge_references.py:227` | `boolean_and_or` | killed |
| `m1_99cdb45824ed603b2b48429308f20b229656dea796908296918869721fa4cf9a` | `tools/knowledge_references.py:228` | `boolean_and_or` | killed |
| `m1_501da23dd6a87856b50fa284af2cc275de65bca16e36452c2b854d745f302216` | `tools/pypi_provenance.py:54` | `remove_not` | killed |
| `m1_fc55f03a26506cd3f2bacf3373c156d52b28ebeee0ab6d1fe239e991f6363e4b` | `tools/pypi_provenance.py:57` | `boolean_literal` | killed |
| `m1_1230489198a02d4fab7189c0242fec3aa9a0e7a1ec6bb91517798320dd4fc533` | `tools/pypi_provenance.py:67` | `compare_eq_ne` | killed |
| `m1_d1698fcb633b25d3cc8cba706c184f786fc21675bcb325d1ce198b80edbcd5c1` | `tools/pypi_provenance.py:71` | `remove_not` | killed |
| `m1_3f39b07b4efcf91633d7b619b40e0d675a5c6a05f26192f252df3ad0fdaa63e6` | `tools/pypi_provenance.py:74` | `loop_break_insert` | killed |
| `m1_70be279254aa916598f13bc3d9fbfccf369cff59266b0cc7080c12e3a5d8f27c` | `tools/pypi_provenance.py:75` | `compare_eq_ne` | killed |
| `m1_c805a93e31d24feb3e8c560e7e08bdae3018c10bbcc22df0cfeaa46605e02932` | `tools/pypi_provenance.py:84` | `remove_not` | killed |
| `m1_722dc9f608a8a0955056cba609bb26be98906b87a17143cc00caf73e1b3ff653` | `tools/pypi_provenance.py:93` | `remove_not` | killed |
| `m1_d3a1e014e82b0af7f88609ab4ebac667f5e849f2581d5583b173d1f3b98aaad0` | `tools/pypi_provenance.py:94` | `boolean_and_or` | killed |
| `m1_a6e9ae0d68b87d8ac93bca2d1b5bde18ded606e1c19ea4901effb5f18e14c25a` | `tools/pypi_provenance.py:95` | `boolean_and_or` | killed |
| `m1_71899ccd3dca8626cec08281bedef723460c29ffe75e49b074507ac594d6b0fe` | `tools/pypi_provenance.py:99` | `loop_break_insert` | killed |
| `m1_3758503c9fa341fd68ab79d9d44d4009eb11f465d6e4d1c59297a40c859f36c1` | `tools/pypi_provenance.py:100` | `compare_eq_ne` | killed |
| `m1_00ddabc5f25b32cb12b31bcbaab49c8c3655289854733b78c1d6997c6aaa6e5b` | `tools/pypi_provenance.py:108` | `none_truthiness` | survived (equivalent) |
| `m1_9219348e0bb49c544ece9dbeacbd69d9eec385fc614368a1f628084ec16efcbd` | `tools/pypi_provenance.py:114` | `boolean_literal` | killed |
| `m1_8e4ff23aef304b46ca152d9ac852076dd2e2a55099f3fddfd67a22a79645c683` | `tools/pypi_provenance.py:122` | `none_truthiness` | survived (equivalent) |
| `m1_224d3db2e0c01c170721abf2d8ad1cd506f63a70bff0487ff13671600621c580` | `tools/pypi_provenance.py:199` | `compare_eq_ne` | killed |
| `m1_9539c5a5ec8c788b0f9194b4bb16bd26662920107fd284097ba7d776af5d5a14` | `tools/pypi_provenance.py:208` | `compare_eq_ne` | killed |
| `m1_cd4b861e496340cd66c7aad8b7d1c4324db5622bdfb4052783536b039556a0e2` | `tools/pypi_provenance.py:211` | `break_continue` | killed |
| `m1_4026ec729ffcf594567e6abce0f151531fc910dbc2e83271b43db3b06043518d` | `tools/pypi_provenance.py:220` | `loop_break_insert` | killed |
| `m1_78878d172a2c361ff62f19efac1878317d6bcb04a74366865fcd9a7e649c471b` | `tools/pypi_provenance.py:221` | `remove_not` | killed |
| `m1_c6ad5f7fa86a57ea61e06b947d8a45e7ab8fdfcd81a938413a1d6ea341458bd1` | `tools/pypi_provenance.py:229` | `loop_break_insert` | killed |
| `m1_44b744008b1830eb2f4c3322f1ed337acbc5173bee528b7d74a4b09db540162c` | `tools/pypi_provenance.py:244` | `none_truthiness` | survived (equivalent) |
| `m1_d7075116f55cabc1ca866a0ef255dd65126ae203dec89372fda3a02f37167b6e` | `tools/pypi_provenance.py:251` | `boolean_and_or` | killed |
| `m1_67414d759a71308eecdab432e4b9825046cdf92aa1dd53e7c1539fbda97ab20a` | `tools/pypi_provenance.py:252` | `boolean_literal` | killed |
| `m1_654a3bde8acdbcfeebcdec37a19d7085a365beddc48f9dcf9ab26b88054758db` | `tools/pypi_provenance.py:255` | `compare_eq_ne` | killed |
| `m1_f6325327f08c02df54979632fb5578c0c724a0c65bc1fae7d1cffca501b456a0` | `tools/pypi_provenance.py:268` | `loop_break_insert` | killed |
| `m1_2584e07f67f9e4d13c889d7ce31c950987d90102edff3d9457477911b6acc0cc` | `tools/pypi_provenance.py:269` | `boolean_literal` | killed |
| `m1_50712107ec8b00ac91ce12d8da265a27a89b13453c587f310712b6e970d9f9b1` | `tools/pypi_provenance.py:282` | `remove_not` | killed |
| `m1_b1f8f6e22b1b7130527e6b0f4271667d965e883eb2c4836ddeab1c0c49d1f254` | `tools/pypi_provenance.py:286` | `generator_empty` | killed |
| `m1_841bc35d56f15cc58474c1529f8e21cc0ebe2b77319b0f16b98cda92b69d4c2d` | `tools/pypi_release.py:26` | `generator_empty` | killed |
| `m1_f40c51ec6ce69eceea494d11d4f34b500ee3e8a124797c9783a160b587b03958` | `tools/pypi_release.py:42` | `compare_eq_ne` | killed |
| `m1_5bd98445b6cdc83b662e9651e426b01f654eeead92d1d0c8061de1b4a3460ab5` | `tools/pypi_release.py:86` | `compare_eq_ne` | killed |
| `m1_67da9d9f1d3f76c8b43d4574f6df65e2fc2929465531a09610ca2642e2ebee96` | `tools/release.py:24` | `remove_not` | killed |
| `m1_8be8f2c0de8c51fe7b787b86274226ecaf94c58e1d7aa89cc3e44835b2f3a937` | `tools/release.py:34` | `remove_not` | killed |
| `m1_46bc754c8e37f19711de79695e06fed75a48e5aa6e55a3bac0bbdcc06136acf2` | `tools/release.py:43` | `none_truthiness` | survived (equivalent) |
| `m1_b1909ec9e1777cb2d9445186e2ea9c2c825e5010615a1e74ae563e1dd0b882f4` | `tools/release_checks.py:18` | `remove_not` | killed |
| `m1_1816a2ea178f387a2af64de8c3f733b915d1f76606cdae8d246f6764fea7db4b` | `tools/release_checks.py:37` | `compare_eq_ne` | killed |
| `m1_fd4313c5895c5f3953dc40bec8a901506656fb212a01006e7a5f4f6b92733a24` | `tools/release_checks.py:51` | `compare_eq_ne` | killed |
| `m1_fa62ebc58b0c9440e01274486869f846303deb6f411b064bae35bb6637b4d90d` | `tools/release_policy.py:39` | `generator_empty` | killed |
| `m1_53502bc67e40a6261dc7230e2a4bd03e7dd3c9c233a740f1cd9f021e5a9a97cd` | `tools/release_prepare.py:22` | `none_truthiness` | survived (equivalent) |
| `m1_8432ea6c5a3ef8ce3ef347136cdfb09a3c82c25102e1ff47a4a92f1d5d34d316` | `tools/release_prepare.py:59` | `remove_not` | killed |
| `m1_12f5ea9a3989fa5ed6db1d60343b3f6052833d1d215092847e7c31e802868c15` | `tools/wheel_smoke.py:18` | `remove_not` | killed |
| `m1_5805ac5e09c2a3ba3b04d468e08c8ef419b5c7c43b1da4d24ecdd0f03070ecf6` | `tools/wheel_smoke.py:26` | `boolean_literal` | killed |
| `m1_5737482843917308f1c3e1da908e0b749cabbe8eabc12855f0d05929cdf0e8ce` | `tools/wheel_smoke.py:55` | `boolean_literal` | killed |
| `m1_40afa67450898e01f66412b6e5b1173c44f9fc3f22c91c7b3e18d2b9ca56b72e` | `tools/release_prepare.py:14` | `string_literal_empty` | killed |
