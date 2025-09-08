// synchronized forecast: temperature
//
let ame_options = {
    series: [],
    chart: {
        id: 'ame',
        group: 'mygrid',
        height: 350,
        type: 'line',
        toolbar: {
            show: false,
        },
        zoom: {
            enabled: false,
        },
    },
    colors: ["#FEB019"],
    stroke: {
        curve: 'smooth',
        width: 2,
    },
    fill: {
        type:'solid',
        opacity: 1,
    },
    yaxis: {
        axisBorder: {
            show: false
        },
        axisTicks: {
            show: false,
        },
        labels: {
            show: true,
            minWidth: 30,
            formatter: function (val) {
                return (val * 100) + " %";
            }
        }
    },
    xaxis: {
        position: 'bottom',
        type: 'datetime',
        axisBorder: {
            show: false
        },
        axisTicks: {
            show: true
        },
        labels: {
            show: true,
        },
    },
    tooltip: {
        enabled: true,
        x: {
            show: true,
            format: 'HH:mm',
        },
        y: {
            formatter: function(value, { series, seriesIndex, dataPointIndex, w }) {
                return Math.round(value * 100) + ' %';
            }
        },
    },
    title: {
        text: 'Air Mass Effect',
        floating: true,
        offsetY: 0,
        align: 'center',
    },
    noData: {
        text: 'Loading...'
    },
    theme: {
        mode: 'dark',
        palette: 'palette1',
        monochrome: {
            enabled: false,
            color: '#255aee',
            shadeTo: 'light',
            shadeIntensity: 0.65
        },
    }
};

let ame = new ApexCharts(document.querySelector("#air_mass_effect"), ame_options);
ame.render();
