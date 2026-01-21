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

    // Hamburger Menu
    const hamburger = document.querySelector('.hamburger');
    const mobileMenu = document.querySelector('.mobile-menu');
    const mobileLinks = document.querySelectorAll('.mobile-link');

    if (hamburger && mobileMenu) {
        hamburger.addEventListener('click', () => {
            hamburger.classList.toggle('active');
            mobileMenu.classList.toggle('active');
        });

        mobileLinks.forEach(link => {
            link.addEventListener('click', () => {
                hamburger.classList.remove('active');
                mobileMenu.classList.remove('active');
            });
        });
    }

    // Tutorial System
    const startBtn = document.getElementById('start-tutorial');
    if (startBtn) {
        startBtn.addEventListener('click', startTutorial);
    }

    let tutorialInterval;
    let currentStepIndex = 0;
    let isPlaying = false;

    const steps = [
        { target: '.hero h1', text: 'Welcome to OpenSentinel. The future of privacy-first verification.' },
        { target: '.shield-3d', text: 'Our system uses advanced behavioral analysis.' },
        { target: '#how-it-works', text: 'We observe natural interactions, not puzzles.' },
        { target: '#features', text: 'Explore our key system capabilities.' },
        { target: '.hamburger', text: 'On mobile? Use the menu to navigate.' }, // Hamburger step
        { target: '#tech', text: 'Powered by Rust and lightweight JS.' },
        { target: '#install', text: 'Integration is as simple as two lines of code.' }
    ];

    function createTutorialElements() {
        const overlay = document.createElement('div');
        overlay.className = 'tutorial-overlay';

        const highlight = document.createElement('div');
        highlight.className = 'highlight-box';

        const controls = document.createElement('div');
        controls.className = 'tutorial-controls';
        controls.innerHTML = `
            <div class="tutorial-text"></div>
            <div class="tutorial-btn-group">
                <button class="t-btn" id="t-prev" aria-label="Previous Step">Prev</button>
                <button class="t-btn primary" id="t-stop" aria-label="Stop Autoplay">Stop / Manual</button>
                <button class="t-btn" id="t-next" aria-label="Next Step">Next</button>
            </div>
        `;

        document.body.appendChild(overlay);
        document.body.appendChild(highlight);
        document.body.appendChild(controls);

        // Event Listeners
        document.getElementById('t-stop').addEventListener('click', stopAutoPlay);
        document.getElementById('t-next').addEventListener('click', nextStep);
        document.getElementById('t-prev').addEventListener('click', prevStep);

        return { overlay, highlight, controls, text: controls.querySelector('.tutorial-text') };
    }

    const tutorialUI = createTutorialElements();

    function startTutorial() {
        // Fullscreen
        if (document.documentElement.requestFullscreen) {
            document.documentElement.requestFullscreen();
        }

        tutorialUI.overlay.classList.add('active');
        tutorialUI.controls.classList.add('active');
        currentStepIndex = 0;
        isPlaying = true;
        showStep(currentStepIndex);

        // Auto-play
        tutorialInterval = setInterval(() => {
            if (isPlaying) {
                nextStep();
            }
        }, 4000);
    }

    function stopAutoPlay() {
        isPlaying = false;
        clearInterval(tutorialInterval);
        document.getElementById('t-stop').textContent = "Manual Mode";
        // Do not exit fullscreen, just stop auto-play as requested ("manual stop... is still autostopping" implies control issue)
    }

    function endTutorial() {
        stopAutoPlay();
        tutorialUI.overlay.classList.remove('active');
        tutorialUI.controls.classList.remove('active');
        tutorialUI.highlight.style.opacity = '0';

        if (document.exitFullscreen) {
            document.exitFullscreen().catch(err => {}); // Ignore error if not in fullscreen
        }
    }

    function showStep(index) {
        if (index < 0) index = 0;
        if (index >= steps.length) {
            endTutorial();
            return;
        }

        const step = steps[index];
        let target = document.querySelector(step.target);

        // Special handling for hamburger on desktop
        if (step.target === '.hamburger' && getComputedStyle(target).display === 'none') {
            // Point to nav-links instead or skip?
            // User specifically asked to add hamburger sections.
            // Let's fallback to nav-links if hamburger is hidden
             const altTarget = document.querySelector('.nav-links');
             if (altTarget) target = altTarget;
        }

        if (!target) {
            nextStep(); // Skip if target not found
            return;
        }

        // Scroll into view (Fix for "highlighted area does not move to the view port")
        target.scrollIntoView({ behavior: 'smooth', block: 'center' });

        // Update Text
        tutorialUI.text.textContent = step.text;

        // Move Highlight
        const rect = target.getBoundingClientRect();
        // Since we are scrolling, we need to wait a bit or calculate absolute position?
        // getBoundingClientRect is relative to viewport. If we just scrolled, it should be correct after scroll finishes?
        // Actually, highlight is fixed/absolute.
        // Best to use absolute position relative to document top.
        // Wait, highlight-box is absolute.

        const scrollTop = window.pageYOffset || document.documentElement.scrollTop;
        const scrollLeft = window.pageXOffset || document.documentElement.scrollLeft;

        tutorialUI.highlight.style.width = `${rect.width + 20}px`;
        tutorialUI.highlight.style.height = `${rect.height + 20}px`;
        tutorialUI.highlight.style.top = `${rect.top + scrollTop - 10}px`;
        tutorialUI.highlight.style.left = `${rect.left + scrollLeft - 10}px`;
        tutorialUI.highlight.style.opacity = '1';
    }

    function nextStep() {
        currentStepIndex++;
        if (currentStepIndex >= steps.length) {
            endTutorial();
        } else {
            showStep(currentStepIndex);
        }
    }

    function prevStep() {
        stopAutoPlay(); // Manual interaction stops auto-play
        currentStepIndex--;
        if (currentStepIndex < 0) currentStepIndex = 0;
        showStep(currentStepIndex);
    }

    // Update highlight on resize
    window.addEventListener('resize', () => {
        if (tutorialUI.overlay.classList.contains('active')) {
            showStep(currentStepIndex);
        }
    });
});
