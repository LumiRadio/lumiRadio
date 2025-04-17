// modular_dice_distribution.js - A flexible tool for dice game distribution analysis

// Core dice rolling function
function rollDice(count) {
    return Array(count).fill(0).map(() => Math.floor(Math.random() * 6) + 1);
  }
  
  // Function to simulate dice rolls with dynamic reward structure
  function simulateDiceGame(diceCount, trials, rewardRanges) {
    const results = {
      wins: 0,
      losses: 0,
      totalWinnings: 0,
      netProfit: 0,
      rewardCounts: {},
      sumCounts: {}
    };
    
    // Sort reward ranges by their min value for proper evaluation
    rewardRanges.sort((a, b) => a.min - b.min);
    
    for (let i = 0; i < trials; i++) {
      // Roll dice
      const dice = rollDice(diceCount);
      const sum = dice.reduce((a, b) => a + b, 0);
      
      // Record sum
      results.sumCounts[sum] = (results.sumCounts[sum] || 0) + 1;
      
      // Find applicable reward for this sum
      let winnings = 0;
      for (const range of rewardRanges) {
        if (sum >= range.min && sum <= range.max) {
          winnings = range.reward;
          break;
        }
      }
      
      if (winnings > 0) {
        results.wins++;
        results.totalWinnings += winnings;
        results.netProfit += winnings - 5; // Subtract cost to play
        results.rewardCounts[winnings] = (results.rewardCounts[winnings] || 0) + 1;
      } else {
        results.losses++;
        results.netProfit -= 5; // Cost to play
      }
    }
    
    // Calculate statistics
    results.winRate = (results.wins / trials) * 100;
    results.lossRate = (results.losses / trials) * 100;
    results.avgWinnings = results.totalWinnings / trials;
    results.avgNetProfit = results.netProfit / trials;
    
    return results;
  }
  
  // Calculate the theoretical expected value based on all possible combinations
  function calculateTheoreticalExpectedValue(diceCount, rewardRanges) {
    // Sort reward ranges by their min value for proper evaluation
    rewardRanges.sort((a, b) => a.min - b.min);
    
    // Calculate the distribution of sums
    const sumDistribution = calculateSumDistribution(diceCount);
    const totalCombinations = Math.pow(6, diceCount);
    
    // Calculate EV using the distribution and rewards
    let ev = 0;
    for (const [sum, count] of Object.entries(sumDistribution)) {
      const sumValue = parseInt(sum);
      const probability = count / totalCombinations;
      
      // Find the reward for this sum
      let reward = 0;
      for (const range of rewardRanges) {
        if (sumValue >= range.min && sumValue <= range.max) {
          reward = range.reward;
          break;
        }
      }
      
      ev += probability * reward;
    }
    
    // Subtract cost to play
    return ev - 5;
  }
  
  // Calculate the distribution of all possible sums for a given number of dice
  function calculateSumDistribution(diceCount) {
    const distribution = {};
    
    function generateCombinations(currentDice, remainingDice, currentSum) {
      if (remainingDice === 0) {
        distribution[currentSum] = (distribution[currentSum] || 0) + 1;
        return;
      }
      
      for (let i = 1; i <= 6; i++) {
        generateCombinations([...currentDice, i], remainingDice - 1, currentSum + i);
      }
    }
    
    generateCombinations([], diceCount, 0);
    return distribution;
  }
  
  // Helper function to get current reward ranges from UI
  function getRewardRangesFromUI() {
    const rangeElements = document.querySelectorAll('.reward-range');
    const ranges = [];
    
    rangeElements.forEach(el => {
      const min = parseInt(el.querySelector('.range-min').value);
      const max = parseInt(el.querySelector('.range-max').value);
      const reward = parseInt(el.querySelector('.range-reward').value);
      
      // Validate inputs
      if (!isNaN(min) && !isNaN(max) && !isNaN(reward)) {
        ranges.push({ min, max, reward });
      }
    });
    
    return ranges;
  }
  
  // Render the simulation results
  function renderResults(results, diceCount, rewardRanges) {
    const resultsDiv = document.getElementById('results');
    
    // Create HTML content
    let html = `<h2>${diceCount}-Dice Simulation Results</h2>`;
    html += `<p><strong>Trials:</strong> ${document.getElementById('trials').value}</p>`;
    html += `<p><strong>Win Rate:</strong> ${results.winRate.toFixed(2)}%</p>`;
    html += `<p><strong>Loss Rate:</strong> ${results.lossRate.toFixed(2)}%</p>`;
    html += `<p><strong>Average Winnings:</strong> ${results.avgWinnings.toFixed(2)} Boondollars</p>`;
    html += `<p><strong>Average Net Profit:</strong> ${results.avgNetProfit.toFixed(2)} Boondollars</p>`;
    
    // Theoretical EV
    const theoreticalEV = calculateTheoreticalExpectedValue(diceCount, rewardRanges);
    html += `<p><strong>Theoretical Expected Value:</strong> ${theoreticalEV.toFixed(4)} Boondollars</p>`;
    
    // Reward distribution
    html += `<h3>Reward Distribution</h3>`;
    html += `<table>
      <tr>
        <th>Reward</th>
        <th>Count</th>
        <th>Percentage</th>
      </tr>`;
    
    const rewardKeys = Object.keys(results.rewardCounts).sort((a, b) => parseInt(a) - parseInt(b));
    for (const reward of rewardKeys) {
      const count = results.rewardCounts[reward];
      const percentage = (count / parseInt(document.getElementById('trials').value)) * 100;
      html += `<tr>
        <td>${reward} Boondollars</td>
        <td>${count}</td>
        <td>${percentage.toFixed(2)}%</td>
      </tr>`;
    }
    html += `</table>`;
    
    // Sum distribution
    html += `<h3>Sum Distribution</h3>`;
    html += `<div style="max-height: 400px; overflow-y: auto;">`;
    html += `<table>
      <tr>
        <th>Sum</th>
        <th>Count</th>
        <th>Percentage</th>
        <th>Reward</th>
      </tr>`;
    
    const sumKeys = Object.keys(results.sumCounts).sort((a, b) => parseInt(a) - parseInt(b));
    for (const sum of sumKeys) {
      const count = results.sumCounts[sum];
      const percentage = (count / parseInt(document.getElementById('trials').value)) * 100;
      
      // Find the reward for this sum
      let reward = 0;
      for (const range of rewardRanges) {
        if (parseInt(sum) >= range.min && parseInt(sum) <= range.max) {
          reward = range.reward;
          break;
        }
      }
      
      html += `<tr>
        <td>${sum}</td>
        <td>${count}</td>
        <td>${percentage.toFixed(2)}%</td>
        <td>${reward} Boondollars</td>
      </tr>`;
    }
    html += `</table>`;
    html += `</div>`;
    
    // Display theoretical distribution
    html += `<h3>Theoretical Sum Distribution</h3>`;
    html += `<div style="max-height: 400px; overflow-y: auto;">`;
    html += `<table>
      <tr>
        <th>Sum</th>
        <th>Combinations</th>
        <th>Probability</th>
        <th>Reward</th>
      </tr>`;
    
    const theoreticalDistribution = calculateSumDistribution(diceCount);
    const totalCombinations = Math.pow(6, diceCount);
    
    const distributions = Object.entries(theoreticalDistribution)
      .map(([sum, count]) => ({ sum: parseInt(sum), count }))
      .sort((a, b) => a.sum - b.sum);
    
    for (const { sum, count } of distributions) {
      const probability = (count / totalCombinations) * 100;
      
      // Find the reward for this sum
      let reward = 0;
      for (const range of rewardRanges) {
        if (sum >= range.min && sum <= range.max) {
          reward = range.reward;
          break;
        }
      }
      
      html += `<tr>
        <td>${sum}</td>
        <td>${count}</td>
        <td>${probability.toFixed(4)}%</td>
        <td>${reward} Boondollars</td>
      </tr>`;
    }
    html += `</table>`;
    html += `</div>`;
    
    // Add the HTML to the results div
    resultsDiv.innerHTML = html;
  }
  
  // Add a new reward range input row
  function addRewardRange(min = "", max = "", reward = "") {
    const container = document.getElementById('reward-ranges');
    const rangeId = Date.now(); // Unique ID for this range
    
    const rangeElement = document.createElement('div');
    rangeElement.className = 'reward-range';
    rangeElement.innerHTML = `
      <div class="reward-row">
        <label>Sum Range:</label>
        <input type="number" class="range-min" placeholder="Min" value="${min}">
        <span>to</span>
        <input type="number" class="range-max" placeholder="Max" value="${max}">
        <label>Reward:</label>
        <input type="number" class="range-reward" placeholder="Boondollars" value="${reward}">
        <button type="button" class="remove-range-btn" onclick="removeRewardRange(this)">✕</button>
      </div>
    `;
    
    container.appendChild(rangeElement);
  }
  
  // Remove a reward range input row
  function removeRewardRange(button) {
    const rangeElement = button.closest('.reward-range');
    rangeElement.parentNode.removeChild(rangeElement);
  }
  
  // Create default reward ranges based on dice count
  function setupDefaultRewardRanges(diceCount) {
    // Clear existing ranges
    document.getElementById('reward-ranges').innerHTML = '';
    
    if (diceCount === 3) {
      // Default 3-dice reward structure
      addRewardRange(3, 10, 0);
      addRewardRange(11, 14, 10);
      addRewardRange(15, 15, 15);
      addRewardRange(16, 16, 20);
      addRewardRange(17, 17, 25);
      addRewardRange(18, 18, 50);
    } else if (diceCount === 4) {
      // Default 4-dice reward structure
      addRewardRange(4, 12, 0);
      addRewardRange(13, 15, 15);
      addRewardRange(16, 18, 25);
      addRewardRange(19, 20, 40);
      addRewardRange(21, 21, 50);
      addRewardRange(22, 22, 75);
      addRewardRange(23, 23, 125);
      addRewardRange(24, 24, 200);
    } else {
      // Generic structure for other dice counts
      const minSum = diceCount;
      const maxSum = diceCount * 6;
      const median = Math.floor((minSum + maxSum) / 2);
      
      addRewardRange(minSum, Math.floor(median * 0.75), 0);
      addRewardRange(Math.floor(median * 0.75) + 1, median, 10);
      addRewardRange(median + 1, Math.floor(median * 1.25), 25);
      addRewardRange(Math.floor(median * 1.25) + 1, maxSum - 1, 50);
      addRewardRange(maxSum, maxSum, 100);
    }
  }
  
  // Run the simulation when the form is submitted
  function runSimulation(event) {
    event.preventDefault();
    
    // Get form values
    const diceCount = parseInt(document.getElementById('dice-count').value);
    const trials = parseInt(document.getElementById('trials').value);
    
    // Get custom reward ranges
    const rewardRanges = getRewardRangesFromUI();
    
    // Validate input
    if (rewardRanges.length === 0) {
      alert('Please add at least one reward range.');
      return;
    }
    
    // Check for overlapping ranges
    for (let i = 0; i < rewardRanges.length; i++) {
      for (let j = i + 1; j < rewardRanges.length; j++) {
        if (
          (rewardRanges[i].min <= rewardRanges[j].max && rewardRanges[i].max >= rewardRanges[j].min) ||
          (rewardRanges[j].min <= rewardRanges[i].max && rewardRanges[j].max >= rewardRanges[i].min)
        ) {
          alert('Reward ranges cannot overlap. Please fix the ranges and try again.');
          return;
        }
      }
    }
    
    // Check for uncovered ranges
    const minPossibleSum = diceCount;
    const maxPossibleSum = diceCount * 6;
    let coveredSums = new Set();
    
    for (const range of rewardRanges) {
      for (let sum = range.min; sum <= range.max; sum++) {
        coveredSums.add(sum);
      }
    }
    
    for (let sum = minPossibleSum; sum <= maxPossibleSum; sum++) {
      if (!coveredSums.has(sum)) {
        alert(`Sum ${sum} is not covered by any reward range. Please ensure all possible sums are covered.`);
        return;
      }
    }
    
    // Show loading indicator
    document.getElementById('results').innerHTML = '<p>Running simulation...</p>';
    
    // Run simulation (with setTimeout to allow UI to update)
    setTimeout(() => {
      const results = simulateDiceGame(diceCount, trials, rewardRanges);
      renderResults(results, diceCount, rewardRanges);
    }, 50);
  }
  
  // Update min/max hints when dice count changes
  function updateSumHints() {
    const diceCount = parseInt(document.getElementById('dice-count').value);
    const minSum = diceCount;
    const maxSum = diceCount * 6;
    
    document.getElementById('sum-range-hint').textContent = 
      `Possible sum range for ${diceCount} dice: ${minSum} to ${maxSum}`;
  }
  
  // Add event listeners once the DOM is loaded
  document.addEventListener('DOMContentLoaded', function() {
    // Set up form submission
    document.getElementById('simulation-form').addEventListener('submit', runSimulation);
    
    // Set up dice count change handler
    const diceCountInput = document.getElementById('dice-count');
    diceCountInput.addEventListener('change', function() {
      updateSumHints();
      setupDefaultRewardRanges(parseInt(this.value));
    });
    
    // Initialize with default settings
    updateSumHints();
    setupDefaultRewardRanges(parseInt(diceCountInput.value));
    
    // Set up add range button
    document.getElementById('add-range-btn').addEventListener('click', function() {
      addRewardRange();
    });
  });