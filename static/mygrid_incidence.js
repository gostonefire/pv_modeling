// combined production and estimated production
//
let incidence_options = {
    series: [],
    chart: {
        id: 'incidence',
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
    colors: ["#008FFB", "#00E396"],
    stroke: {
        curve: 'smooth',
        width: [2,2],
    },
    fill: {
        type:'solid',
        opacity: [1, 1],
    },
    yaxis: {
        min: 0,
        max: 90,
        reversed: true,
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
                return val + " deg";
            }
        }
    },
    xaxis: {
        position: 'bottom',
        type: 'datetime',
        axisBorder: {
            show: false,
        },
        axisTicks: {
            show: true,
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
                return Math.round(value * 10) / 10 + ' deg';
            }
        },
    },
    title: {
        text: 'Sun Incidence',
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


let incidence = new ApexCharts(document.querySelector("#incidence"), incidence_options);
incidence.render();
