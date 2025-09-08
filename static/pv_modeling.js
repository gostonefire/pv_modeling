function loadScriptSequentially(file) {
    return new Promise((resolve, reject) => {
        const newScript = document.createElement('script');
        newScript.setAttribute('src', file);
        newScript.setAttribute('async', 'true');

        newScript.onload = () => {
            resolve(); // Resolve the promise
        };
        newScript.onerror = () => {
            displayMessage(`Error loading script: ${file}`, 'error');
            reject(new Error(`Error loading script: ${file}`));
        };

        document.head.appendChild(newScript);
    });
}

function updateData() {
    let year = $("#year").text();
    let month = $("#month").text();
    let day = $("#day").text();

    let panel_power = $("#panel_power").text();
    let panel_slope = $("#panel_slope").text();
    let panel_east_azm = $("#panel_east_azm").text();
    let panel_temp_red = $("#panel_temp_red").text();

    let tau = $("#tau").text();
    let tau_down = $("#tau_down").text();
    let k_gain = $("#k_gain").text();

    let iam_factor = $("#iam_factor").text();

    let start_azm = $("#start_azm").text();
    let start_elv = $("#start_elv").text();
    let stop_azm = $("#stop_azm").text();
    let stop_elv = $("#stop_elv").text();

    let url = '/get_data?year=' + year + '&month=' + month + '&day=' + day +
        '&panel_power=' + panel_power + '&panel_slope=' + panel_slope + '&panel_east_azm=' + panel_east_azm +
        '&panel_temp_red=' + panel_temp_red + '&tau=' + tau + '&tau_down=' + tau_down + '&k_gain=' + k_gain + '&iam_factor=' + iam_factor +
        '&start_azm=' + start_azm + '&start_elv=' + start_elv + '&stop_azm=' + stop_azm + '&stop_elv=' + stop_elv;

    $.getJSON(url, function(resp, textStatus, jqXHR) {
        production.updateSeries(resp.prod_diagram);
        incidence.updateSeries(resp.incidence_diagram);
        temp.updateSeries(resp.temp_diagram);
        ame.updateSeries([resp.ame_diagram]);
    });
}

function getData() {
    $.getJSON('/get_start', function(resp, textStatus, jqXHR) {
        console.log(resp.params);
        $("#year").text(resp.params.year);
        $("#month").text(resp.params.month);
        $("#day").text(resp.params.day);

        $("#panel_power").text(resp.params.panel_power);
        $("#panel_slope").text(resp.params.panel_slope);
        $("#panel_east_azm").text(resp.params.panel_east_azm);
        $("#panel_temp_red").text(resp.params.panel_temp_red);

        $("#tau").text(resp.params.tau);
        $("#tau_down").text(resp.params.tau_down);
        $("#k_gain").text(resp.params.k_gain);

        $("#iam_factor").text(resp.params.iam_factor);

        $("#start_azm").text(resp.params.start_azm);
        $("#start_elv").text(resp.params.start_elv);
        $("#stop_azm").text(resp.params.stop_azm);
        $("#stop_elv").text(resp.params.stop_elv);

        production.updateSeries(resp.prod_diagram);
        incidence.updateSeries(resp.incidence_diagram);
        temp.updateSeries(resp.temp_diagram);
        ame.updateSeries([resp.ame_diagram]);

    });
}

loadScriptSequentially('locale_se.js')
    .then(() => loadScriptSequentially('mygrid_prod.js'))
    .then(() => loadScriptSequentially('mygrid_incidence.js'))
    .then(() => loadScriptSequentially('mygrid_temp.js'))
    .then(() => loadScriptSequentially('mygrid_ame.js'))
    .then(() => {
        getData();
    })
    .catch(error => displayMessage(error.message, 'error'));



function displayMessage(message, type) {
    console.log(message, type);
}
