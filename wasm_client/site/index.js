/* global uPlot */
import init, { start_log_stream } from "../pkg/wasm_client.js";

async function run() {
  await init();
  start_log_stream();

  // uPlot state
  const chartData = [[], [], []];

  const opts = {
    title: "Log Traffic",
    width: 800,
    height: 300,
    scales: { x: { time: false } },
    series: [
      {},
      {
        label: "Total Logs",
        stroke: "#6366f1",
        width: 2,
      },
      {
        label: "Errors",
        stroke: "#ef4444",
        width: 2,
        fill: "rgba(239, 68, 68, 0.1)",
      },
    ],
    axes: [
      {
        stroke: "#9ca3af",
        grid: {
          stroke: "#374151",
          width: 1,
        },
      },
      {
        stroke: "#9ca3af",
        grid: {
          stroke: "#374151",
          width: 1,
        },
      },
    ],
    series: [
      {},
      {
        label: "Total Logs",
        stroke: "#6366f1",
        width: 2,
      },
      {
        label: "Errors",
        stroke: "#ef4444",
        width: 2,
        fill: "rgba(239, 68, 68, 0.1)",
      },
    ],
  };

  const uplot = new uPlot(
    opts,
    chartData,
    document.getElementById("chart-container"),
  );

  window.updateUPlot = (xArr, yTotal, yErrors) => {
    if (!uplot) {
      console.error("❌ uPlot instance is missing!");
      return;
    }

    try {
      uplot.setData([xArr, yTotal, yErrors]);
    } catch (e) {
      console.error("❌ uPlot Draw Error:", e);
    }
  };
}

run();
