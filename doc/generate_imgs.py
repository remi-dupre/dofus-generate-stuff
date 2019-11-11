#!/usr/bin/env python3
from math import exp
import matplotlib
import matplotlib.pyplot as plt
import numpy as np


sigm_tex = r"$f: x \mapsto 1 \div \left(1 + \exp\left(\frac{-4 (x - target + width)}{width}\right)\right)$"


def sigm(target, width, x):
    return 1 / (1 + exp(-4 * (x - target + width) / width))


def draw_sigm(target, width, label):
    x_vals = np.linspace(target - 6 * width, target + width, 1000)
    y_vals = [sigm(target, width, x) for x in x_vals]
    plt.plot(x_vals, y_vals, label=sigm_tex, color="#ff4444")
    plt.axvline(x=target, linestyle=":", label="target")
    plt.axvspan(
        target - 3 * width / 2,
        target - width / 2,
        color="lightblue",
        label=r"± $\frac{width}{2}$",
    )
    plt.xlabel(label)
    plt.legend(frameon=False)


def draw_prod_sigm(target1, width1, label1, target2, width2, label2):
    x1_vals = np.linspace(target1 - 6 * width1, target1 + width1, 1000)
    x2_vals = np.linspace(target2 - 6 * width2, target2 + width2, 1000)
    vals = [
        [sigm(target1, width1, x1) * sigm(target2, width2, x2) for x1 in x1_vals]
        for x2 in x2_vals
    ]
    plt.pcolormesh(
        x1_vals, x2_vals, vals, norm=matplotlib.colors.LogNorm(), cmap="tab20b"
    )

    plt.xlabel(label1)
    plt.ylabel(label2)
    plt.colorbar()


#  ---
plt.figure(figsize=(15, 4))

plt.subplot(121)
draw_sigm(11, 1, "AP")

plt.subplot(122)
draw_sigm(1500, 100, "Intelligence")

plt.savefig("img/sigmoid1D.svg")

#  ---
plt.close()
draw_prod_sigm(11, 1, "AP", 1500, 100, "Intelligence")
plt.savefig("img/sigmoid2D.png")
