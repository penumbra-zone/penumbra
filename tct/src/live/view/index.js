d3.json(window.location.href + "/dot")
    .then(function(response) {
        d3.select("#graph").graphviz()
            .renderDot(response.graph);
    }).catch(err => d3.select("#graph").text(err).style("color: #AA0000"));