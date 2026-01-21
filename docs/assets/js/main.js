document.addEventListener('DOMContentLoaded', () => {
    // 3D Tilt Effect
    const tiltElements = document.querySelectorAll('.tilt-element');

    tiltElements.forEach(el => {
        el.addEventListener('mousemove', handleMove);
        el.addEventListener('mouseleave', handleLeave);
        el.style.transition = 'transform 0.1s';
    });

    function handleMove(e) {
        const el = this;
        const height = el.clientHeight;
        const width = el.clientWidth;

        const rect = el.getBoundingClientRect();
        const xVal = e.clientX - rect.left;
        const yVal = e.clientY - rect.top;

        // Calculate rotation (max 15 degrees)
        const yRotation = 15 * ((xVal - width / 2) / width);
        const xRotation = -15 * ((yVal - height / 2) / height);

        // Apply transform
        const transformString = `perspective(1000px) scale(1.05) rotateX(${xRotation}deg) rotateY(${yRotation}deg)`;
        el.style.transform = transformString;

        // Parallax for inner content if exists
        const inner = el.querySelector('img') || el.querySelector('.card-icon');
        if (inner) {
            inner.style.transform = `translateZ(50px)`;
        }
    }

    function handleLeave() {
        this.style.transform = 'perspective(1000px) scale(1) rotateX(0) rotateY(0)';
        this.style.transition = 'transform 0.5s ease-out';
    }

    // Navbar Scroll Effect
    const navbar = document.querySelector('.navbar');
    window.addEventListener('scroll', () => {
        if (window.scrollY > 50) {
            navbar.style.background = 'rgba(5, 5, 5, 0.8)';
            navbar.style.backdropFilter = 'blur(20px)';
        } else {
            navbar.style.background = 'transparent';
            navbar.style.backdropFilter = 'blur(10px)';
        }
    });

    // Glitch Text Randomization
    const glitchText = document.querySelector('.glitch');
    if (glitchText) {
        setInterval(() => {
            const r1 = Math.random() * 10;
            const r2 = Math.random() * 10;
            glitchText.style.setProperty('--after-top', `${r1}px`);
            glitchText.style.setProperty('--before-top', `${r2}px`);
        }, 2000);
    }

    // Auto-update Year
    const yearSpan = document.getElementById('current-year');
    if (yearSpan) {
        yearSpan.textContent = new Date().getFullYear();
    }
});
