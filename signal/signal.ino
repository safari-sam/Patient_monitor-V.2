#include <DHT.h>

// --- PINS ---
#define DHTPIN 2       // Temp sensor on Pin 2
#define DHTTYPE DHT11
#define MOTION_PIN 3   // Motion sensor on Pin 3
#define SOUND_PIN A0   // Sound sensor on Analog Pin 0

DHT dht(DHTPIN, DHTTYPE);

// --- SOUND DETECTION VARIABLES ---
volatile int peakSound = 0;        // Highest sound detected since last reset
unsigned long lastResetTime = 0;   // When we last reset the peak
const int RESET_INTERVAL = 1000;   // Reset peak every 1 second

// Threshold for "spike" detection during continuous monitoring
const int SPIKE_THRESHOLD = 100;   // Adjust based on your sensor

void setup() {
  Serial.begin(9600);
  dht.begin();
  pinMode(MOTION_PIN, INPUT);
  
  // Set up Timer1 interrupt for continuous sound sampling
  // This runs in background even during delays
  cli();  // Disable interrupts during setup
  
  TCCR1A = 0;
  TCCR1B = 0;
  TCNT1 = 0;
  
  // Set compare match register for ~1000Hz sampling (every 1ms)
  OCR1A = 249;  // 16MHz / (64 * 250) = 1000Hz
  
  TCCR1B |= (1 << WGM12);   // CTC mode
  TCCR1B |= (1 << CS11) | (1 << CS10);  // 64 prescaler
  TIMSK1 |= (1 << OCIE1A);  // Enable timer compare interrupt
  
  sei();  // Enable interrupts
}

// This interrupt runs ~1000 times per second, sampling sound continuously
ISR(TIMER1_COMPA_vect) {
  int sample = analogRead(SOUND_PIN);
  
  // We care about deviation from center (~512 for 5V reference)
  // Calculate absolute deviation from center
  int deviation = abs(sample - 512);
  
  // Track the peak deviation since last reset
  if (deviation > peakSound) {
    peakSound = deviation;
  }
}

void loop() {
  // 1. Read temperature and motion
  float t = dht.readTemperature();
  int motion = digitalRead(MOTION_PIN);
  
  // 2. Get the peak sound level captured by the interrupt
  //    and reset for next cycle
  int sound = peakSound;
  peakSound = 0;  // Reset for next reading
  
  // 3. Safety check: if temp is invalid, send 0
  if (isnan(t)) t = 0.0;

  // 4. Send Data: "Temp,Motion,Sound"
  Serial.print(t);
  Serial.print(",");
  Serial.print(motion);
  Serial.print(",");
  Serial.println(sound);

  delay(1000);
}
