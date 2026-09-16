"""Qualify paired cost only when all sixteen outputs equal verified whole PCM."""
import argparse
import json
from pathlib import Path
import statistics

from verification_common import sha


def read(path):
    return json.loads(path.read_text(encoding='utf-8'))


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--root',type=Path,required=True)
    parser.add_argument('--output',type=Path,required=True)
    args = parser.parse_args()
    assert not args.output.exists()
    paths = dict(job=args.root/'prepared-reuse-job-v1.json',native=args.root/'prepared-reuse-v1/report.json',
                 process=args.root/'prepared-reuse-v1.process.json')
    job,native,process = [read(path) for path in paths.values()]
    assert native['status'] == process['status'] == 'complete' and process['exit_code'] == 0
    assert native['job_sha256'] == process['job_sha256'] == sha(paths['job'])
    assert job['binary_sha256'] == process['binary_sha256'] == sha(Path(job['binary']))
    assert sha(Path(job['specification'])) == job['specification_sha256']
    assert sha(Path(job['baseline_summary'])) == job['baseline_summary_sha256']
    baseline = read(Path(job['baseline_summary']))
    assert baseline['status'] == 'complete' and len(native['rows']) == 16
    assert len(native['source_preparation']) == len(job['cases']) == 4
    result = dict(status='complete',scope='Paired existing production APIs in an opt-in release unit probe; no app adoption or response claim',
        inputs={key:dict(path=str(path),sha256=sha(path)) for key,path in paths.items()},
        specification_sha256=job['specification_sha256'],baseline_summary_sha256=job['baseline_summary_sha256'],
        script_sha256=sha(Path(__file__)),process=process,rows=native['rows'],cases=[])
    for case in job['cases']:
        assert sha(Path(case['prepared_path'])) == case['prepared_sha256']
        assert sha(Path(case['expected_path'])) == case['expected_sha256']
        reference = next(row for row in baseline['rows'] if row['id'] == case['id'])
        assert reference['sha256'] == case['expected_sha256']
        rows = [row for row in native['rows'] if row['id'] == case['id']]
        assert {(row['round'],row['reused_measurements']) for row in rows} == {(0,False),(0,True),(1,False),(1,True)}
        for row in rows:
            assert row['exact_pcm'] and row['expected_sha256'] == case['expected_sha256']
            assert (row['frames'],row['rate'],row['channels']) == (reference['frames'],48000,2)
            assert row['lufs'] == reference['lufs'] and row['peak'] == reference['peak'] <= -1
            assert row['gain'] == reference['gain']
        fresh = [row['finalization_s'] for row in rows if not row['reused_measurements']]
        reused = [row['finalization_s'] for row in rows if row['reused_measurements']]
        preparation = next(row for row in native['source_preparation'] if row['id'] == case['id'])
        result['cases'].append(dict(id=case['id'],decode_s=preparation['decode_s'],prepare_s=preparation['prepare_s'],
            fresh_observations_s=fresh,reused_observations_s=reused,
            fresh_mean_s=statistics.mean(fresh),reused_mean_s=statistics.mean(reused),
            incremental_ratio=statistics.mean(fresh)/statistics.mean(reused),
            two_fresh_evaluations_s=sum(fresh),prepare_plus_two_reused_evaluations_s=preparation['prepare_s']+sum(reused)))
    args.output.parent.mkdir(parents=True,exist_ok=True)
    args.output.write_text(json.dumps(result,indent=2,allow_nan=False)+'\n',encoding='utf-8')
    print(json.dumps(result['cases'],indent=2))


if __name__ == '__main__':
    main()
