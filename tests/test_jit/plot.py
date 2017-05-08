import sys, json
import matplotlib.pyplot as plt


def plot(result_dic):
    fig = plt.figure(1, figsize=(9, 6))
    ax = fig.add_subplot(111)
    width = 0.1

    colors = ['#718c00',
              '#eab700',
              '#f5871f',
              '#c82829',
              '#3e999f',
              '#4271ae',
              '#8959a8',
              '#1d1f21']

    all_targets = ('cpython', 'pypy', 'pypy_nojit', 'rpy_c', 'rpy_mu', 'c', 'mu')
    targets = tuple(k for k in all_targets if k in result_dic)
    data = [(tgt, result_dic[tgt]['average'], result_dic[tgt]['std_dev'], result_dic[tgt]['slowdown'])
            for tgt in targets]
    data.sort(key=lambda (tgt, avg, std, sd): avg)
    for i, (tgt, avg, std_dev, slowdown) in enumerate(data):
        ax.bar(width / 2 + width * i, avg, width, color=colors[i], yerr=std_dev, label=tgt)
        ax.text(width / 2 + width * i + 0.01, avg, "%.6f" % avg, color='#1d1f21', fontweight='bold')
        ax.text(width * (i + 1), avg - std_dev, "%.6f" % std_dev, color='#1d1f21', fontweight='bold')
        ax.text(width * (i + 1) - 0.02, avg / 2, "%.3fx" % slowdown, color='#1d1f21', fontweight='bold')
    # plt.legend(loc=2)
    plt.xticks([width * (i + 1) for i in range(len(targets))], [tgt for (tgt, _, _, _) in data])
    plt.title("%(test_name)s with input size %(input_size)d" % result_dic)
    plt.show()


def test_plot():
    # plot(perf_quicksort(1000, 20))
    with open('result_quicksort.json', 'r') as fp:
        plot(json.load(fp))

if __name__ == '__main__':
    with open(sys.argv[1], 'r') as fp:
        plot(json.load(fp))
