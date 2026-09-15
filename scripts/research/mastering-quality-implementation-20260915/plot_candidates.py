"""Standalone scientific figure from retained C0 measurements, no audio reads."""
import argparse
import json
from pathlib import Path
import matplotlib
matplotlib.use('Agg')
import matplotlib.pyplot as plt


def main():
    parser=argparse.ArgumentParser();parser.add_argument('--report',type=Path,required=True)
    parser.add_argument('--output',type=Path,required=True);a=parser.parse_args()
    assert not a.output.exists();a.output.mkdir(parents=True)
    data=json.loads(a.report.read_text());rows=data['rows']
    fig,axes=plt.subplots(3,2,figsize=(11,10),sharex='col',layout='constrained')
    styles={'control':('x','#202731','Current processing'),
            'single':('D','#bd3b35','Single rule'),
            'attenuate':('^','#157860','Attenuate-only'),
            'bounded':('o','#326cad','Bounded grid')}
    for col,target in enumerate([-14,-9]):
        selected=[r for r in rows if r['target']==target]
        for row,(key,label) in enumerate([('lufs','Delivered LUFS'),('crest','Whole-file crest (dB)'),
                                           ('section_contrast_change_db','Section contrast change (dB)')]):
            ax=axes[row,col]
            grid=sorted([r for r in selected if r['policy'] in ['single','bounded']],key=lambda r:r['extra_drive_db'])
            ax.plot([r['extra_drive_db'] for r in grid],[r['metrics'][key] for r in grid],color='#326cad',alpha=.45,lw=1)
            for policy,(marker,color,name) in styles.items():
                points=[r for r in selected if r['policy']==policy]
                ax.scatter([r['extra_drive_db'] for r in points],[r['metrics'][key] for r in points],
                           marker=marker,color=color,label=name,s=48,zorder=4)
            baseline=target if row==0 else data['source']['crest'] if row==1 else 0
            ax.axhline(baseline,ls='--',lw=1,color='#737b83',label='Target' if row==0 else 'Source')
            ax.grid(alpha=.15);ax.set_ylabel(label);ax.spines[['top','right']].set_visible(False)
            if row==0:ax.set_title(f'{target} LUFS target · −1 dBTP ceiling',weight='bold')
            if row==2:ax.set_xlabel('Additional experimental drive (dB)')
    axes[0,0].legend(fontsize=8,loc='lower right')
    fig.suptitle('C0 owner-source experiment\nDrive, target attainment and source-relative dynamics',fontsize=15,weight='bold')
    fig.supxlabel('Complete float outputs checked with corrected FFT16 and float SOXR16.\nCrest alone does not establish sound preference. LUFS summaries resolve to 0.1 LU.',fontsize=9)
    fig.savefig(a.output/'drive-comparison.png',dpi=180)
    fig.savefig(a.output/'drive-comparison.svg')
    plt.close(fig)


if __name__=='__main__':main()
