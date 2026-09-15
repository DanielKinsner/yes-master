"""Experimental signed far-tail estimate with a geometric-series remainder.

For a source node centred at c and target block centred at t0, d=t0-c:
1/(d+u-v) = sum(k=0..P) (-1)^k (u-v)^k/d^(k+1) + remainder.
If q=(target_radius+source_radius)/abs(d)<1, the absolute remainder is
<= node_L1/abs(d) * q^(P+1)/(1-q). Accept nodes only at q<=1/2.
Store source moments once; translate them into a target polynomial and evaluate
that polynomial across the block. This is an exact algebraic remainder bound,
not an empirical reserve. Floating error is a separate qualification concern.
"""
from math import comb
import numpy as np

ORDER=20
BINOM=np.array([[comb(i,j) if j<=i else 0 for j in range(ORDER+1)] for i in range(ORDER+1)])


class Node:
    def __init__(self,x,start,span,block):
        self.start=start;self.stop=min(len(x),start+span)
        self.centre=start+span/2;self.radius=span/2
        self.children=[];self.moments=np.zeros(ORDER+1)
        if span==block:
            indices=np.arange(start,self.stop)
            values=x[start:self.stop]*(-1.)**indices
            v=(indices-self.centre)/self.radius
            weight=np.ones(len(values))
            for k in range(ORDER+1):
                self.moments[k]=np.sum(values*weight)
                weight*=v
            self.l1=float(np.sum(np.abs(values)))
        else:
            for a in [start,start+span//2]:
                if a<len(x):self.children.append(Node(x,a,span//2,block))
            self.l1=sum(c.l1 for c in self.children)
            for child in self.children:
                shift=(child.centre-self.centre)/self.radius
                scale=child.radius/self.radius
                for k in range(ORDER+1):
                    self.moments[k]+=sum(BINOM[k,m]*shift**(k-m)*scale**m*child.moments[m] for m in range(k+1))

    def add_far(self,a,b,centre,radius,coefficients):
        if self.l1==0 or (self.start>=a and self.stop<=b):return 0.
        d=centre-self.centre
        q=(radius+self.radius)/abs(d) if d else np.inf
        if (self.stop<=a or self.start>=b) and q<=.5:
            source_ratio=self.radius/d
            target_ratio=-radius/d
            for j in range(ORDER+1):
                coefficients[j]+=target_ratio**j/d*sum(
                    BINOM[m+j,j]*source_ratio**m*self.moments[m] for m in range(ORDER+1-j))
            return self.l1/abs(d)*q**(ORDER+1)/(1-q)
        assert self.children, 'local span must cover complete source blocks'
        return sum(child.add_far(a,b,centre,radius,coefficients) for child in self.children)


def tree(x,block):
    span=block
    while span<len(x):span*=2
    return Node(x,0,span,block)


def far_polynomial(root,a,b,start,core):
    coefficients=np.zeros(ORDER+1)
    remainder=root.add_far(a,b,start+core/2,core/2,coefficients)
    return coefficients,remainder
