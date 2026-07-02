/* Morph mockup shell — tiny vanilla helpers shared by every screen.
   No framework, no build, no network. */

/* Materialize a set of elements (adds staggered .materialize classes). */
function materialize(selector) {
  document.querySelectorAll(selector).forEach((el, i) => {
    el.classList.remove('hidden', 'dissolving');
    el.classList.add('materialize', 'd' + Math.min(i, 5));
  });
}

/* Dissolve elements, then hide them once the animation finishes. */
function dissolve(selector) {
  document.querySelectorAll(selector).forEach((el) => {
    el.classList.remove('materialize', 'd1', 'd2', 'd3', 'd4', 'd5');
    el.classList.add('dissolving');
    el.addEventListener('animationend', () => {
      el.classList.add('hidden');
      el.classList.remove('dissolving');
    }, { once: true });
  });
}

/* Step sequencer: elements tagged data-step="N" appear when goToStep(N) runs.
   Elements tagged data-step-hide="N" dissolve at that step. */
let __step = 0;
function goToStep(n) {
  __step = n;
  document.querySelectorAll('[data-step]').forEach((el) => {
    const at = Number(el.dataset.step);
    if (at === n) materializeEl(el);
    else if (at > n) { el.classList.add('hidden'); }
  });
  document.querySelectorAll('[data-step-hide]').forEach((el) => {
    if (Number(el.dataset.stepHide) === n) dissolveEl(el);
  });
  document.querySelectorAll('[data-step-label]').forEach((el) => {
    el.textContent = el.dataset['label' + n] || el.textContent;
  });
}
function nextStep() { goToStep(__step + 1); }

function materializeEl(el) {
  el.classList.remove('hidden', 'dissolving');
  el.classList.add('materialize');
}
function dissolveEl(el) {
  el.classList.remove('materialize');
  el.classList.add('dissolving');
  el.addEventListener('animationend', () => {
    el.classList.add('hidden');
    el.classList.remove('dissolving');
  }, { once: true });
}

/* "Why" toggles on rail cards. */
document.addEventListener('click', (e) => {
  const t = e.target.closest('.why-toggle');
  if (t) {
    const card = t.closest('.rail-card');
    card.classList.toggle('open');
    t.textContent = card.classList.contains('open') ? 'hide why' : 'why?';
  }
});

/* Presence indicator state helper. */
function presence(state) {
  document.querySelectorAll('.presence').forEach((p) => {
    p.classList.remove('thinking', 'asking', 'dreaming');
    if (state && state !== 'idle') p.classList.add(state);
  });
}

/* Dismissal lesson picker: swap a card's body for the three-way lesson choice. */
function askLesson(btn) {
  const card = btn.closest('.rail-card');
  card.innerHTML =
    '<h4>Which lesson?</h4>' +
    '<p>So the mind learns the right thing.</p>' +
    '<div class="dismiss-row">' +
    '<button onclick="learned(this, \'Okay — I\\u2019ll try again another time.\')">not now</button>' +
    '<button onclick="learned(this, \'Understood — I won\\u2019t suggest this again.\')">never</button>' +
    '<button onclick="learned(this, \'Noted — I misread what you wanted.\')">wrong tool</button>' +
    '</div>';
}
function learned(btn, msg) {
  const card = btn.closest('.rail-card');
  card.innerHTML = '<p style="color:var(--ink-dim)">' + msg + '</p>';
  setTimeout(() => dissolveEl(card), 1200);
}
