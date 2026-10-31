#![no_main]
use libfuzzer_sys::fuzz_target;
use taloon::{inspection::{Inspection,AcquisitionResult},evidence::{Evidence,SidecarAssociation},model::{ContentHash,EvaluationDate}};
fuzz_target!(|data: &[u8]| {
    let bytes=include_bytes!("../corpus/wheel/example.whl");
    let hash=ContentHash::digest(bytes);
    let lock=format!("version=1\nrevision=5\n[[package]]\nname='example'\nversion='1.0'\nsource={{registry='https://example.test/simple'}}\nwheels=[{{url='https://example.test/a.whl',hash='{hash}'}},{{url='https://example.test/b.whl',hash='{hash}'}}]");
    let required=data.first().is_some_and(|b|b&1!=0);
    let mut policy="[tool.taloon]\nschema-version=1\nallow=['MIT']".to_owned();
    if required{policy.push_str(&format!("\n[[tool.taloon.clarifications]]\nartifact-hash='{hash}'\nexpression='MIT'\nreason='fuzz'"));}
    let inspection=Inspection::from_documents(&lock,&policy,EvaluationDate::parse("2026-10-31").unwrap()).unwrap();
    let meta=b"Metadata-Version: 2.4\nName: example\nVersion: 1.0\nLicense-Expression: MIT\n\n";
    let build=||{
        let mut results=Vec::new();
        for (n,request) in inspection.requests().iter().enumerate(){
            let slot=data.get(n+1).copied().unwrap_or(0)%4;
            if slot==0{continue;}
            let evidence=if slot==1{Evidence::from_sidecar(request,SidecarAssociation::new(hash,hash,ContentHash::digest(meta)).unwrap(),meta).unwrap()}else{Evidence::from_archive(request,bytes).unwrap()};
            results.push(AcquisitionResult::observed(request.id(),evidence.clone()));
            if slot==3{results.push(AcquisitionResult::observed(request.id(),evidence));}
        }
        results
    };
    let a=inspection.evaluate(build());let mut reversed=build();reversed.reverse();let b=inspection.evaluate(reversed);
    assert_eq!(serde_json::to_value(&a).unwrap(),serde_json::to_value(&b).unwrap());
    let incomplete=(0..2).any(|n|{let slot=data.get(n+1).copied().unwrap_or(0)%4;slot==0||slot==3||(slot==1&&required)});
    if incomplete{assert_eq!(a.exit_code(false),2);}
});
