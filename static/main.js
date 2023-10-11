function getCellValue(tr, index = 0) {
    const cell = tr.children[index]
    if (cell.childElementCount == 1) {
        const child = cell.firstElementChild
        if (child instanceof HTMLTimeElement && child.dateTime) {
            return child.dateTime
        } else if (child instanceof HTMLDataElement && child.value) {
            return child.value
        }
    }
    return cell.innerText || cell.textContent;
}

function rowComparator(trA, trB, index = 0) {
    let valueA = getCellValue(trA, index);
    let valueB = getCellValue(trB, index);
    if (!isNaN(valueA) && !isNaN(valueB)) {
        return valueA - valueB
    }
    return valueA.localeCompare(valueB, undefined, {numeric: true});
}

function sortColumn(th) {
    const dir = th.dataset["sortedDirection"] || "ascending";  // use aria-sorted
    [...th.parentElement.children].forEach(header => delete header.dataset["sortedDirection"]);
    th.dataset["sortedDirection"] = dir === "ascending" ? "descending" : "ascending";

    const index = [...th.parentElement.children].indexOf(th)

    Array.from(th.closest('table').querySelectorAll('tbody tr'))
        .sort((trA, trB) => rowComparator(trA, trB, index) * (dir == "ascending" ? 1 : -1))
        .forEach(tr => tr.parentElement.appendChild(tr) );
}

document.querySelectorAll('th[data-sortable]').forEach(
    th => th.addEventListener('click', () => sortColumn(th))
);
