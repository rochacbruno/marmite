
const menuToggle = document.getElementById('menu-toggle');
const headerMenu = document.getElementById('header-menu');

menuToggle.addEventListener('click', function () {
  headerMenu.classList.toggle('active');
});