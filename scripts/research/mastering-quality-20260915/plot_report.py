from pathlib import Path
import sys,json,csv
P=Path(__file__).resolve().parent
sys.path.insert(0,str(P/'pylibs'))
import numpy as np
import matplotlib
matplotlib.use('Agg')
import matplotlib.pyplot as plt
from scipy.io import wavfile
E=P/'compact-evidence'
rows=list(csv.DictReader((E/'music-results.csv').open()))
ids=['coat','piano','funk','metal','baby','aphelion','rich','imaginal']
labels=["It's a coat","Solo piano","Funkorama","Metalmania","Baby Bird*","Aphelion","Rich*","Imaginal*"]
plt.rcParams.update({'font.family':'DejaVu Sans','axes.spines.top':False,'axes.spines.right':False,'font.size':10})
fig,axs=plt.subplots(1,2,figsize=(13,5.5),layout='constrained');y=np.arange(8)
for ax,name,label,color in [(axs[0],'control_t14','Current','#697586'),(axs[0],'drive_t14','Selected drive','#177d69')]:
    data=[next(r for r in rows if r['source']==id and r['name']==name) for id in ids]
    ax.scatter([float(r['crest']) for r in data],y,label=label,c=color,s=65,zorder=3)
for i,id in enumerate(ids):
    a=next(r for r in rows if r['source']==id and r['name']=='control_t14');b=next(r for r in rows if r['source']==id and r['name']=='drive_t14')
    axs[0].plot([float(a['crest']),float(b['crest'])],[i,i],color='#bac4c8',zorder=1)
    axs[1].plot([float(a['lufs']),float(b['lufs'])],[i,i],color='#bac4c8',zorder=1)
    axs[1].scatter(float(a['lufs']),i,color='#697586',s=65);axs[1].scatter(float(b['lufs']),i,color='#177d69',s=65)
for ax in axs:ax.set_yticks(y,labels);ax.invert_yaxis();ax.grid(axis='x',alpha=.2)
axs[0].set(xlabel='Peak-to-RMS crest (dB)',title='Processing tradeoff');axs[0].legend(loc='upper right')
axs[1].axvline(-14,color='#c05e30',ls='--',lw=1);axs[1].set(xlabel='Delivered integrated loudness (LUFS)',title='Delivery is a separate criterion')
fig.suptitle('Same -14 LUFS request, different amounts of processing',fontsize=16,fontweight='bold')
fig.text(.5,-.12,'Universal 75, -1 dBTP requested. Frozen absolute trim grid. *Constructed stem mixes. Higher crest is not a quality score.\nImaginal exceeds the requested peak ceiling; Rich and Imaginal miss loudness target. All eight sources are retained.',ha='center',fontsize=9)
fig.savefig(E/'drive-vs-delivery.png',dpi=160,bbox_inches='tight');plt.close(fig)
center=json.loads((E/'centered-validation.json').read_text());old=json.loads((E/'robustness.json').read_text());fig,axes=plt.subplots(2,2,figsize=(11,6),layout='constrained')
for ax,id,label in zip(axes.flat,['coat','piano','rich','imaginal'],["It's a coat","Solo piano","Rich (stem sum)","Imaginal (stem sum)"]):
    a=sorted([r for r in old if r['source']==id and r['target']==-14 and r['mode']=='control'],key=lambda r:r['gain']);b=sorted([r for r in center if r['source']==id and r['target']==-14],key=lambda r:r['gain'])
    ax.plot([r['gain'] for r in a],[r['measure']['crest_db'] for r in a],'o-',color='#697586',label='Current');ax.plot([r['gain'] for r in b],[r['independent']['crest'] for r in b],'o-',color='#177d69',label='Source-centred search')
    ax.set(title=label,xlabel='Input file gain change (dB)',ylabel='Output crest (dB)',xticks=[-12,-6,0]);ax.grid(alpha=.2)
axes[0,0].legend();fig.suptitle('Source level should not accidentally choose the mastering density',fontsize=15,fontweight='bold');fig.text(.5,-.12,'Exploratory follow-up, -14 target: all 12 centred outputs reach target within 0.2 LU and pass the independent ceiling check.\nAt -9, input-gain bounds and peak-estimator failures remain; this is not ready for production adoption.',ha='center',fontsize=9)
fig.savefig(E/'input-level-robustness.png',dpi=160,bbox_inches='tight');plt.close(fig)
fig,axes=plt.subplots(1,2,figsize=(12,4.8),layout='constrained');x=np.linspace(-1.5,1.5,900)
for a,label,color in [(0,'Bypass','#697586'),(1e-8,'Current, near zero','#b27a31'),(.0715,'Current Universal 75','#b24a4a'),(-.0715,'Continuous prototype','#177d69')]:
    y=x if a==0 else np.tanh(x*(1+2*a))/np.tanh(1+2*a) if a>0 else np.tanh(x*(-2*a))/(-2*a)
    axes[0].plot(x,y,label=label,color=color)
axes[0].set(title='A small positive amount is not a small wet blend',xlabel='Input sample',ylabel='Output sample');axes[0].legend(fontsize=8);axes[0].grid(alpha=.2)
for mode,color in [('current','#b24a4a'),('fixed','#177d69')]:
    sr,w=wavfile.read(P/f'src-check/48000_44100_2_100_{mode}.wav');axes[1].plot(np.arange(940,973),w[940:973,0],'.-',color=color,label=mode)
axes[1].set(title='One incorrect SRC sample: isolated correction',xlabel='Output frame (48 → 44.1 kHz)',ylabel='Amplitude');axes[1].legend();axes[1].grid(alpha=.2)
fig.savefig(E/'mechanisms.png',dpi=160,bbox_inches='tight')
print('three evidence plots saved')
