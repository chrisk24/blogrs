
function bannerClick(){
	let icon = document.getElementById("banner-expand-btn");
	let sidebar = document.getElementById("sidebar")
	
	console.log(sidebar.className);
	if (sidebar.className.includes("sidebar-closed")){
		icon.innerHTML = '&lt;';
		sidebar.className = sidebar.className.replace("sidebar-closed", "sidebar-open");
	} else {
		icon.innerHTML = '&gt;';
		sidebar.className = sidebar.className.replace("sidebar-open", "sidebar-closed");
	}
}
