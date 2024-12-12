function callAPI(endpoint) {
    fetch(`/v1/${endpoint}`, {
        method: "POST"
    }).then(response=>response.json())
    .then(data=>{console.log(data);})
}

function readQR() {
    var result = document.getElementById("result");
    fetch(`/qrcode`, {
        method: "POST",
        signal: AbortSignal.timeout(3000),
        headers: {
            "Cache-Control": "no-cache, no-store, must-revalidate",  // Prevent caching
            "Pragma": "no-cache",
        },
    })
    .then(response => {
        if (!response.ok) throw new Error(`HTTP error: ${response.status}`);
        return response.text();
    })
    .then(data => {
        result.innerText = data;
        result.style.display="";
    })
    .catch(error => {
        console.error("Fetch error:", error.message);
        result.innerText = `${error.message}`;
        result.style.display = "";
    });
}

function closeWindow() {
    var xpwindow = document.getElementById("window-body");
    xpwindow.style.display = 'none';
}

function openWindow() {
    var xpwindow = document.getElementById("window-body");
    xpwindow.style.display = '';
}

function xPressed() {
    var result = document.getElementById("result");
    result.innerText = "Sorry, you cannot close this program :)";
    result.style.display = "";
}

function disableButtons() {
    var buttons = document.getElementById("driving").querySelectorAll("button");
    buttons.forEach((x, i) => x.disabled=true);

    var buttons = document.getElementById("lift").querySelectorAll("button");
    buttons.forEach((x, i) => x.disabled=true);
}

function enableButtons() {
    var buttons = document.getElementById("driving").querySelectorAll("button");
    buttons.forEach((x, i) => x.disabled=false);

    var buttons = document.getElementById("lift").querySelectorAll("button");
    buttons.forEach((x, i) => x.disabled=false);
}

function setAPIStatus() {
    var healthy = fetch("/v1/health", {
        signal: AbortSignal.timeout(3000),
    })
        .then((response)=>{
            if (response.status == 200) {
                enableButtons();
                document.getElementById("apistatus").innerText = "API Status: Online"
            } else {
                disableButtons();
                document.getElementById("apistatus").innerText = "API Status: Offline"
            }
        }
    ).catch(error=>{});
}

function livefeedOffline() {    
    var static_src = "static/tv_static.gif"
    var livefeed = document.getElementById("livefeed");
    
    var buttons = document.getElementById("camera").querySelectorAll("button");
    buttons.forEach((x, i) => x.disabled=true);

    if (livefeed.src != static_src) {
        livefeed.src="static/tv_static.gif"
    }
}

function livefeedOnline() {
    var stream_src = "/stream"
    var livefeed = document.getElementById("livefeed");

    var buttons = document.getElementById("camera").querySelectorAll("button");
    buttons.forEach((x, i) => x.disabled=false);

    if (livefeed != stream_src) {
        livefeed.src="/stream"
    }
}

function setLivefeed() {
    var healthy = fetch("/stream", {
        signal: AbortSignal.timeout(3000),
    })
        .then((response)=>{
            if (response.status == 502) {
                livefeedOffline();
            } else {
                livefeedOnline();
            }
        }
    );
}

window.onload = function() {
    setAPIStatus();
    setLivefeed();
    setInterval(function() {
        setAPIStatus();
        setLivefeed();
    }, 5000);
}
