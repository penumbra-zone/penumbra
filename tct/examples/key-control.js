function keyControl() {
    let actions = [];
    let pendingCount = null;

    const queries = {
        'c': ["post", 'insert?witness=forget'],
        'C': ["post", 'insert?witness=keep'],
        'b': ["post", 'end-block'],
        'B': ["post", 'insert-block-root'],
        'e': ["post", 'end-epoch'],
        'E': ["post", 'insert-epoch-root'],
        'f': ["post", 'forget'],
        'n': ["post", 'new'],
        'r': ["get", 'root'],
    };

    const digits = new Set(['0', '1', '2', '3', '4', '5', '6', '7', '8', '9']);
    const stops = new Set(['Escape', 'Backspace', 'Enter', 'q', 'Q']);

    window.addEventListener('keydown', event => {
        let key = event.key;

        if (stops.has(key)) {
            event.preventDefault();
            actions = [];
            pendingCount = null;
            display(key);
            display("");
        } else if (key === 'c' && event.ctrlKey) {
            event.preventDefault();
            actions = [];
            pendingCount = null;
            display('^C');
            display("");
        } else if (key in queries) {
            event.preventDefault();

            // How many of this operation to do
            let count = 1;
            if (pendingCount !== null) {
                count = pendingCount;
                // We've used this pending count, so reset it
                pendingCount = null;
            }

            // Enqueue the operation
            if (actions.length === 0 || actions[0].key !== key) {
                actions.unshift({
                    key: key,
                    count: count,
                })
            } else {
                actions[0].count += count;
            }

            // Ensure the operation is done
            doAction();
        } else if (digits.has(key)) {
            event.preventDefault();

            // Add this digit to the pending count
            if (pendingCount === null) {
                pendingCount = 0;
            }
            pendingCount = pendingCount * 10 + parseInt(key);
            display(pendingCount);
        }
    });

    function doAction() {
        if (actions.length === 0) {
            display("");
            return;
        }

        let action = actions[actions.length - 1];
        let key = action.key;

        if (action.count === 0) {
            // This action is done
            display("");
            actions.pop();
            return;
        } else {
            if (action.count > 1) {
                display(action.count + " " + key);
            } else {
                display(key);
            }
            // Decrement the count
            action.count -= 1;
        }

        let url = window.location.origin + '/' + queries[key][1];

        d3.text(url, {method: queries[key][0]}).then(() => {
            // Continue doing actions until none are left to do
            doAction();
        }).catch(error => {
            // If there was an error, stop the loop
            console.log(error);
        });
    }

    // Set up the visual feedback box
    d3.select('#graph').insert("div").attr("id", "message");
    const message = d3.select('#message');
    const messageColor = "rgba(50, 50, 100, 0.5)";
    message.style("width", "100%");
    message.style("position", "absolute");
    message.style("bottom", "0");
    message.style("text-align", "center");
    message.style("font-size", "200pt");
    message.style("font-family", "Courier New");
    message.style("margin-bottom", "50pt");
    message.style("color", messageColor);

    function display(string) {
        if (string.length === 0) {
            message.transition()
                .duration(500)
                .style("color", "rgba(100, 100, 100, 0.0)")
                .end()
                .then(() => message.text(string));
        } else {
            message.text(string);
            message.style("color", messageColor);
        }
    }
}

keyControl();